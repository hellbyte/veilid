use super::*;

pub(in super::super) struct PrepareBeginInboundTransactionContext {
    pub opaque_record_key: OpaqueRecordKey,
    pub descriptor: Arc<SignedValueDescriptor>,
    pub want_descriptor: bool,
    pub signing_member_id: MemberId,
    pub subkey_count: usize,
}

pub(in super::super) enum PrepareBeginInboundTransactionResult {
    Done(InboundTransactBeginResult),
    Continue(PrepareBeginInboundTransactionContext),
}

pub(in super::super) struct PrepareEndInboundTransactionContext {
    pub opaque_record_key: OpaqueRecordKey,
    pub transaction_id: InboundTransactionId,
    pub opt_begin_snapshot: Option<Arc<RecordSnapshot>>,
}

pub(in super::super) enum PrepareEndInboundTransactionResult {
    Done(InboundTransactCommandResult),
    Continue(PrepareEndInboundTransactionContext),
}

pub(in super::super) struct PrepareCommitInboundTransactionContext<D: RecordDetail> {
    pub transaction_id: InboundTransactionId,
    pub opt_commit_action: Option<CommitAction<D>>,
}

pub(in super::super) enum PrepareCommitInboundTransactionResult<D: RecordDetail> {
    Done(InboundTransactCommandResult),
    Continue(PrepareCommitInboundTransactionContext<D>),
}

impl<D> RecordStoreInner<D>
where
    D: RecordDetail,
{
    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn lookup_inbound_transaction_id(
        &mut self,
        raw_id: u64,
    ) -> VeilidAPIResult<Option<InboundTransactionId>> {
        self.inbound_transactions.lookup_id(raw_id)
    }

    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn prepare_begin_inbound_transaction(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        opt_descriptor: Option<SignedValueDescriptor>,
        want_descriptor: bool,
        signing_member_id: MemberId,
    ) -> VeilidAPIResult<PrepareBeginInboundTransactionResult> {
        // Get descriptor
        let opt_existing_descriptor =
            self.with_record(opaque_record_key, |record| record.descriptor())?;
        let descriptor = match opt_existing_descriptor {
            Some(x) => x,
            None => {
                // Needs descriptor
                let Some(descriptor) = opt_descriptor.map(Arc::new) else {
                    return Ok(PrepareBeginInboundTransactionResult::Done(
                        InboundTransactBeginResult::NeedDescriptor,
                    ));
                };

                descriptor
            }
        };
        let owner = descriptor.owner();
        let schema = descriptor.schema()?;
        let subkey_count = schema.subkey_count();

        // Make a closure to check for member vs anonymous
        let owner_member_id = self.storage_manager().generate_member_id(&owner)?;
        let member_check = Box::new(move |signer: &MemberId| {
            owner_member_id == *signer || schema.is_member(signer.ref_value())
        });

        // See if this record has a transaction already for this signer
        // And tabulate the number of total transactions to check against limits
        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        // Calculate transaction limits
        let mut transaction_count = 0;
        let is_member = member_check(&signing_member_id);

        if let Some(active_transaction_list) = self.inbound_transactions.get_mut(&rtk) {
            // Total up the number of transactions for this key
            for t in active_transaction_list.transactions() {
                // See if this transactions should be counted toward any limits
                let count_transaction = if is_member {
                    // If the signer is a member of the schema, then consider the total per-signer key
                    t.signing_member_id() == &signing_member_id
                } else {
                    // If the signer is not a member of the schema, the check if this transacation is an anonymous transacation and contributes to per-record key total
                    !member_check(t.signing_member_id())
                };

                if count_transaction {
                    transaction_count += 1;
                }
            }
        }

        // Validate limit
        let transaction_limit = if is_member {
            // One transaction per schema-member signer per record
            self.unlocked_inner.limits.member_transaction_limit
        } else {
            self.unlocked_inner.limits.public_transaction_limit
        };
        if transaction_count >= transaction_limit {
            return Ok(PrepareBeginInboundTransactionResult::Done(
                InboundTransactBeginResult::TransactionUnavailable,
            ));
        }

        Ok(PrepareBeginInboundTransactionResult::Continue(
            PrepareBeginInboundTransactionContext {
                opaque_record_key: opaque_record_key.clone(),
                descriptor,
                want_descriptor,
                signing_member_id,
                subkey_count,
            },
        ))
    }

    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn finish_begin_inbound_transaction(
        &mut self,
        begin_context: PrepareBeginInboundTransactionContext,
        opt_snapshot: Option<Arc<RecordSnapshot>>,
    ) -> VeilidAPIResult<InboundTransactBeginResult> {
        let PrepareBeginInboundTransactionContext {
            opaque_record_key,
            descriptor,
            want_descriptor,
            signing_member_id,
            subkey_count,
        } = begin_context;

        // Make transaction expiration timestamp
        let expiration = Timestamp::now().later(self.unlocked_inner.limits.transaction_timeout);

        // Get the snapshot seqs to return
        let seqs = if let Some(snapshot) = &opt_snapshot {
            snapshot.seqs()
        } else {
            vec![ValueSeqNum::NONE; subkey_count]
        };

        // Create a new transaction
        let transaction_id = self.inbound_transactions.allocate(
            &opaque_record_key,
            expiration,
            signing_member_id,
            descriptor.clone(),
            opt_snapshot,
        )?;

        // Return the result
        Ok(InboundTransactBeginResult::Success(TransactBeginSuccess {
            transaction_id,
            expiration,
            opt_descriptor: want_descriptor.then_some(descriptor),
            seqs,
        }))
    }

    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn prepare_end_inbound_transaction(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        transaction_id: InboundTransactionId,
    ) -> VeilidAPIResult<PrepareEndInboundTransactionResult> {
        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        let opt_begin_snapshot = {
            let Some(active_transaction_list) = self.inbound_transactions.get_mut(&rtk) else {
                return Ok(PrepareEndInboundTransactionResult::Done(
                    InboundTransactCommandResult::InvalidTransaction,
                ));
            };

            // If the lock id is ours, then someone tried to 'end' twice, so invalidate the transaction.
            // We don't lock here yet to allow for an unchanged transaction to finish regardless of an
            // existing lock held by another transaction
            if active_transaction_list.is_locked_by(transaction_id) {
                self.inbound_transactions
                    .remove_transaction(transaction_id)
                    .unwrap_or_else(veilid_log_err!(self));
                veilid_log!(self debug "{}","ended twice");
                return Ok(PrepareEndInboundTransactionResult::Done(
                    InboundTransactCommandResult::InvalidTransaction,
                ));
            }

            // Get the inbound transaction if it is still valid
            let Some(inbound_transaction) = active_transaction_list.get(transaction_id) else {
                // Nothing to drop
                return Ok(PrepareEndInboundTransactionResult::Done(
                    InboundTransactCommandResult::InvalidTransaction,
                ));
            };

            // If there's no changes, we can just quit early with a zero expiration to indicate no commit or rollback is necessary
            // No lock check is required here because
            if !inbound_transaction.has_changed_subkeys() {
                self.inbound_transactions
                    .remove_transaction(transaction_id)
                    .unwrap_or_else(veilid_log_err!(self));
                return Ok(PrepareEndInboundTransactionResult::Done(
                    InboundTransactCommandResult::Success(TransactCommandSuccess {
                        expiration: Default::default(),
                        opt_seqs: Default::default(),
                        opt_subkey: Default::default(),
                        opt_value: Default::default(),
                    }),
                ));
            }

            // Get begin snapshot for comparison
            let begin_snapshot = inbound_transaction.snapshot();

            // Try to obtain the lock now
            // If there is any other lock, we can't lock this ourselves
            if let Err(e) = active_transaction_list.lock(transaction_id) {
                self.inbound_transactions
                    .remove_transaction(transaction_id)
                    .unwrap_or_else(veilid_log_err!(self));
                veilid_log!(self debug "{}: {}","end failed",e);
                return Ok(PrepareEndInboundTransactionResult::Done(
                    InboundTransactCommandResult::InvalidTransaction,
                ));
            }

            begin_snapshot
        };

        Ok(PrepareEndInboundTransactionResult::Continue(
            PrepareEndInboundTransactionContext {
                opaque_record_key: opaque_record_key.clone(),
                transaction_id,
                opt_begin_snapshot,
            },
        ))
    }

    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn finish_end_inbound_transaction(
        &mut self,
        end_context: PrepareEndInboundTransactionContext,
        res_opt_end_snapshot: VeilidAPIResult<Option<Arc<RecordSnapshot>>>,
    ) -> VeilidAPIResult<InboundTransactCommandResult> {
        let PrepareEndInboundTransactionContext {
            opaque_record_key,
            transaction_id,
            opt_begin_snapshot,
        } = end_context;

        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        let opt_end_snapshot = match res_opt_end_snapshot {
            Ok(v) => v,
            Err(e) => {
                let _ = self
                    .inbound_transactions
                    .try_remove_transaction(transaction_id)
                    .inspect_err(veilid_log_err!(self));
                veilid_log!(self debug "{}: {}","end snapshot failed",e);
                return Ok(InboundTransactCommandResult::InvalidTransaction);
            }
        };

        // If the snapshot doesn't validate then the transaction is not valid
        let Some(active_transaction_list) = self.inbound_transactions.get_mut(&rtk) else {
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        };

        if opt_begin_snapshot.as_ref().map(|s| s.seqs()) != opt_end_snapshot.map(|s| s.seqs()) {
            let _ = self
                .inbound_transactions
                .try_remove_transaction(transaction_id)
                .inspect_err(veilid_log_err!(self));
            veilid_log!(self debug "{}","end snapshot mismatch");
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        }

        // Everything is valid, we can end the transaction successfully as long as it still exists
        let Some(inbound_transaction) = active_transaction_list.get_mut(transaction_id) else {
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        };

        // Give the user another timeout bump to allow for commit
        let expiration = Timestamp::now().later(self.unlocked_inner.limits.transaction_timeout);

        // Record new expiration
        inbound_transaction.update_expiration(expiration);

        Ok(InboundTransactCommandResult::Success(
            TransactCommandSuccess {
                expiration,
                opt_seqs: Default::default(),
                opt_subkey: Default::default(),
                opt_value: Default::default(),
            },
        ))
    }

    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn prepare_commit_inbound_transaction<C: FnOnce() -> D>(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        transaction_id: InboundTransactionId,
        make_record_detail: C,
    ) -> VeilidAPIResult<PrepareCommitInboundTransactionResult<D>> {
        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        let transaction = {
            let Some(active_transaction_list) = self.inbound_transactions.get_mut(&rtk) else {
                return Ok(PrepareCommitInboundTransactionResult::Done(
                    InboundTransactCommandResult::InvalidTransaction,
                ));
            };

            // If there is a record lock then it better be ours
            // * If there is no record lock, then this commit is out of order and this transaction should be dropped
            // * If the lock id is ours, then we can commit
            // * If the lock id is not ours, then this commit is out of order and this transaction should be dropped
            if !active_transaction_list.is_locked_by(transaction_id) {
                self.inbound_transactions
                    .remove_transaction(transaction_id)
                    .unwrap_or_else(veilid_log_err!(self));
                veilid_log!(self debug "{}","bad lock state");
                return Ok(PrepareCommitInboundTransactionResult::Done(
                    InboundTransactCommandResult::InvalidTransaction,
                ));
            }

            // Get the inbound transaction if it is still valid
            let Some(inbound_transaction) = active_transaction_list.get_mut(transaction_id) else {
                apibail_internal!("inbound transaction missing even though it is locked");
            };

            // If there's no changes, the transaction should have been dropped by 'end' and not locked and we shouldnt get here
            if !inbound_transaction.has_changed_subkeys() {
                apibail_internal!("no changes in locked transaction");
            }

            inbound_transaction.clone()
        };

        // See if we have a remote record already or not
        if !self.contains_record(opaque_record_key) {
            // record didn't exist, make it
            let cur_ts = Timestamp::now();
            let record = Record::<D>::new(cur_ts, transaction.descriptor(), make_record_detail())?;
            self.new_record(opaque_record_key.clone(), record)?;
        };

        // Apply all changes
        let watch_update_mode = InboundWatchUpdateMode::UpdateAll;
        let subkey_values = transaction.changed_subkeys().collect::<Vec<_>>();
        let opt_commit_action = self
            .set_subkeys_single_record(opaque_record_key, &subkey_values, &watch_update_mode)
            .inspect_err(veilid_log_err!(
                self,
                "set_subkeys_single_record failed in commit"
            ))?;

        Ok(PrepareCommitInboundTransactionResult::Continue(
            PrepareCommitInboundTransactionContext {
                transaction_id,
                opt_commit_action,
            },
        ))
    }

    #[instrument(level = "trace", target = "stor", skip_all, err)]
    pub fn finish_commit_inbound_transaction(
        &mut self,
        commit_context: PrepareCommitInboundTransactionContext<D>,
    ) -> VeilidAPIResult<InboundTransactCommandResult> {
        let PrepareCommitInboundTransactionContext {
            transaction_id,
            opt_commit_action: _,
        } = commit_context;

        // Drop transaction and lock now that we're done
        if !self
            .inbound_transactions
            .try_remove_transaction(transaction_id)
            .inspect_err(veilid_log_err!(self))?
        {
            veilid_log!(self debug "missing transaction id");
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        }

        return Ok(InboundTransactCommandResult::Success(
            TransactCommandSuccess {
                expiration: Default::default(),
                opt_seqs: Default::default(),
                opt_subkey: Default::default(),
                opt_value: Default::default(),
            },
        ));
    }

    pub fn rollback_inbound_transaction(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        transaction_id: InboundTransactionId,
    ) -> VeilidAPIResult<InboundTransactCommandResult> {
        // See if this transaction is still valid
        if !self.inbound_transactions.check_id(
            transaction_id,
            &RecordTableKey {
                record_key: opaque_record_key.clone(),
            },
        ) {
            veilid_log!(self debug "mismatched transaction id");
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        }

        // Rollback just needs to drop the transaction wherever it is
        if !self
            .inbound_transactions
            .try_remove_transaction(transaction_id)
            .inspect_err(veilid_log_err!(self))?
        {
            veilid_log!(self debug "missing transaction id");
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        }

        Ok(InboundTransactCommandResult::Success(
            TransactCommandSuccess {
                expiration: Default::default(),
                opt_seqs: Default::default(),
                opt_subkey: Default::default(),
                opt_value: Default::default(),
            },
        ))
    }

    pub fn inbound_transaction_get(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        transaction_id: InboundTransactionId,
        opt_subkey: Option<ValueSubkey>,
    ) -> VeilidAPIResult<InboundTransactCommandResult> {
        // See if this transaction is still valid
        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        // If the transaction is still active and not ended/locked
        let Some(active_transaction_list) = self.inbound_transactions.get_mut(&rtk) else {
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        };

        if active_transaction_list.is_locked_by(transaction_id) {
            self.inbound_transactions
                .remove_transaction(transaction_id)
                .unwrap_or_else(veilid_log_err!(self));
            veilid_log!(self debug "{}","get after end");
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        }

        // Get the inbound transaction if it is still valid
        let Some(inbound_transaction) = active_transaction_list.get_mut(transaction_id) else {
            // Nothing to drop
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        };

        // Check if subkey is specified
        let opt_value = if let Some(subkey) = opt_subkey {
            // Ensure subkey is within bounds
            if subkey > inbound_transaction.descriptor().schema()?.max_subkey() {
                self.inbound_transactions
                    .remove_transaction(transaction_id)
                    .unwrap_or_else(veilid_log_err!(self));
                return Ok(InboundTransactCommandResult::InvalidArguments);
            }

            // Get value to return
            if let Some(snapshot) = inbound_transaction.snapshot() {
                snapshot.subkey_value_data(subkey)?
            } else {
                None
            }
        } else {
            // Subkey not specified is just a 'keepalive'
            None
        };

        // Give the user another timeout bump to allow for more commands
        let expiration = Timestamp::now().later(self.unlocked_inner.limits.transaction_timeout);

        // Record new expiration
        inbound_transaction.update_expiration(expiration);

        Ok(InboundTransactCommandResult::Success(
            TransactCommandSuccess {
                expiration,
                opt_seqs: Default::default(),
                opt_subkey,
                opt_value,
            },
        ))
    }

    pub fn inbound_transaction_set(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        transaction_id: InboundTransactionId,
        subkey: ValueSubkey,
        value: Arc<SignedValueData>,
    ) -> VeilidAPIResult<InboundTransactCommandResult> {
        // See if this transaction is still valid
        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        // If the transaction is still active and not ended/locked
        let Some(active_transaction_list) = self.inbound_transactions.get_mut(&rtk) else {
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        };

        if active_transaction_list.is_locked_by(transaction_id) {
            self.inbound_transactions
                .remove_transaction(transaction_id)
                .unwrap_or_else(veilid_log_err!(self));
            veilid_log!(self debug "{}","set after end");
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        }

        // Get the inbound transaction if it is still valid
        let Some(inbound_transaction) = active_transaction_list.get_mut(transaction_id) else {
            // Nothing to drop
            return Ok(InboundTransactCommandResult::InvalidTransaction);
        };

        // Ensure subkey is within bounds
        if subkey > inbound_transaction.descriptor().schema()?.max_subkey() {
            self.inbound_transactions
                .remove_transaction(transaction_id)
                .unwrap_or_else(veilid_log_err!(self));
            return Ok(InboundTransactCommandResult::InvalidArguments);
        }

        // Get value to compare against
        let opt_existing_value = if let Some(snapshot) = inbound_transaction.snapshot() {
            snapshot.subkey_value_data(subkey)?
        } else {
            None
        };

        // If the proposed sequence number is newer, then return no value
        let opt_value = if value.value_data().seq()
            > opt_existing_value
                .as_ref()
                .map(|x| x.value_data().seq())
                .unwrap_or_default()
        {
            // Mark the subkey as changed
            inbound_transaction.add_changed_subkey(subkey, value);

            None
        } else {
            // Mark the subkey as unchanged
            inbound_transaction.remove_changed_subkey(subkey);

            // Return the existing value
            opt_existing_value
        };

        // Give the user another timeout bump to allow for more commands
        let expiration = Timestamp::now().later(self.unlocked_inner.limits.transaction_timeout);

        // Record new expiration
        inbound_transaction.update_expiration(expiration);

        Ok(InboundTransactCommandResult::Success(
            TransactCommandSuccess {
                expiration,
                opt_seqs: Default::default(),
                opt_subkey: Some(subkey),
                opt_value,
            },
        ))
    }

    /// See if any inbound transactions have expired and clear them out
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub fn drop_expired_inbound_transactions(&mut self) {
        let now = Timestamp::now_non_decreasing();
        let registry = self.registry();
        let debug_logger = |id| {
            veilid_log!(registry debug "dropped expired transaction: {}", id);
        };
        let error_logger = |e| {
            veilid_log!(registry error "error in drop_expired_transactions: {}", e);
        };

        self.inbound_transactions
            .remove_expired_transactions(now, &debug_logger, &error_logger);
    }
}
