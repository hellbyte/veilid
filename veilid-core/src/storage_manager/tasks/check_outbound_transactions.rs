use super::*;

impl StorageManager {
    // Check if client-side transactions on opened records have expired
    //#[instrument(level = "trace", target = "stor", skip_all, err)]
    pub(super) fn check_outbound_transactions_task_routine(
        &self,
        _stop_token: StopToken,
        _last_ts: Timestamp,
        _cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let mut expired_transaction_handles = vec![];
        let mut keepalive_transaction_params_list = vec![];

        {
            let inner = &mut *self.inner.lock();

            let cur_ts = Timestamp::now_non_decreasing();

            let otm = &mut inner.outbound_transaction_manager;

            for (transaction_handle, outbound_transaction_state) in otm.transactions() {
                let expired = outbound_transaction_state
                    .opt_expiration()
                    .map(|expiration| expiration < cur_ts)
                    .unwrap_or(true);
                if expired {
                    expired_transaction_handles.push(transaction_handle.clone());
                    continue;
                }

                match otm.prepare_transact_keepalive_params(transaction_handle.clone()) {
                    Ok(v) => {
                        if v.is_empty() {
                            // Nothing to do here
                        } else {
                            for params in v {
                                keepalive_transaction_params_list
                                    .push((transaction_handle.clone(), params));
                            }
                        }
                    }
                    Err(e) => {
                        veilid_log!(self debug "Not preparing keepalives for {}: {}", transaction_handle, e);
                    }
                }
            }
        };

        // Expire all transactions first
        for expired_transaction_handle in expired_transaction_handles {
            // Don't drop expired records that are currently being worked on
            if let Some(records_lock) = self.record_lock_table.try_lock_records(
                expired_transaction_handle.keys().to_vec(),
                StorageManagerRecordLockPurpose::TransactDrop,
            ) {
                let registry = self.registry();
                let drop_fut = async move {
                    let this = registry.storage_manager();

                    veilid_log!(this debug "Dropping expired transaction: {}", expired_transaction_handle);

                    let opt_background_tokens = {
                        let inner = &mut *this.inner.lock();
                        let otm = &mut inner.outbound_transaction_manager;
                        otm.drop_transaction(expired_transaction_handle)
                            .map(|x| x.into_background_tokens())
                    };

                    if let Some(background_tokens) = opt_background_tokens {
                        Self::wait_for_background_tokens(background_tokens).await;
                    }

                    drop(records_lock);
                };

                self.background_operation_processor.add_future(drop_fut);
            }
        }

        // Send all keepalives in a background task if they aren't already being processed
        {
            let inner = &mut *self.inner.lock();
            for (keepalive_transaction_handle, keepalive_transaction_params) in
                keepalive_transaction_params_list
            {
                let opaque_record_key = keepalive_transaction_params.opaque_record_key.clone();

                let state = match inner
                    .outbound_transaction_manager
                    .get_transaction_state_mut(&keepalive_transaction_handle)
                {
                    Ok(v) => v,
                    Err(e) => {
                        veilid_log!(self debug "Dropping transaction keepalive because the transaction is gone: handle={} record={}: {}", keepalive_transaction_handle, opaque_record_key, e);
                        continue;
                    }
                };

                // If we're already doing this keepalive, bail
                if inner
                    .active_transaction_keepalives
                    .contains(&keepalive_transaction_params.opaque_record_key)
                {
                    continue;
                }

                // Peek the record lock so we can do the keepalive safely without blocking any other subkey operations
                // This also ensures that the end/commit waits for this keepalive to finish so we don't have the server-side transaction state wrong
                let Some(peek_lock) = self
                    .record_lock_table
                    .try_peek_lock(opaque_record_key.clone())
                else {
                    // If we can't get the peek lock, then we probably aren't in the 'begin' state, because only subkey
                    // locks happen for gets and sets at that point. Just bail and try again a second later on the next tick
                    // if this happens
                    veilid_log!(self debug "Skipping keepalive as record lock could not be peeked: handle={} record={}", keepalive_transaction_handle, opaque_record_key);
                    continue;
                };

                // Now that we've got the peek lock, start the keepalive action
                inner
                    .active_transaction_keepalives
                    .insert(keepalive_transaction_params.opaque_record_key.clone());

                // Add the background task stop token to this transaction's drop wait list
                let stop_source = StopSource::new();
                let stop_token = stop_source.token();
                state.add_background_token(stop_token);

                // Add the keepalive to the background processor
                let registry = self.registry();
                self.background_operation_processor.add_future(async move {
                    let this = registry.storage_manager();

                    let fut = async {
                        {
                            let inner = &mut *this.inner.lock();
                            let otm = &mut inner.outbound_transaction_manager;
                            let Ok(transaction_state) = otm.get_transaction_state(&keepalive_transaction_handle) else {
                                veilid_log!(this debug "Dropping transaction keepalive because the transaction is gone: handle={} record={}", keepalive_transaction_handle, opaque_record_key);
                                return;
                            };
                            let Some(stage) = transaction_state.stage_consensus() else {
                                veilid_log!(this debug "Dropping transaction keepalive because the transaction has no state: handle={} record={}", keepalive_transaction_handle, opaque_record_key);
                                return;
                            };
                            match stage.stage {
                                OutboundTransactionStage::Begin => {
                                    // Still good to send
                                }
                                OutboundTransactionStage::End
                                | OutboundTransactionStage::Failed
                                | OutboundTransactionStage::Rollback
                                | OutboundTransactionStage::Commit => {
                                    veilid_log!(this debug "Dropping transaction keepalive because the transaction stage is no longer BEGIN: handle={} record={}", keepalive_transaction_handle, opaque_record_key);
                                    return;
                                }
                            }
                        }

                        veilid_log!(this debug "Send transaction keepalive to {}", opaque_record_key);

                        let result = match this.outbound_transact_command(keepalive_transaction_params).await {
                            Ok(v) => v,
                            Err(e) => {
                                veilid_log!(this debug "Not sending transaction keepalive for (handle={}, key={}): {}", keepalive_transaction_handle, opaque_record_key, e);
                                return;
                            }
                        };

                        let inner = &mut *this.inner.lock();
                        let otm = &mut inner.outbound_transaction_manager;
                        if let Err(e) = otm.record_transact_keepalive_results(keepalive_transaction_handle.clone(), vec![result]) {
                            veilid_log!(this debug "Not recording transaction keepalive results for (handle={}, key={}): {}", keepalive_transaction_handle, opaque_record_key, e);
                        }
                    };

                    fut.await;

                    {
                        drop(stop_source);

                        let inner = &mut *this.inner.lock();

                        // Move the stop source in here and drop it when we're done
                        if let Ok(transaction_state) = inner
                            .outbound_transaction_manager
                            .get_transaction_state_mut(&keepalive_transaction_handle)
                        {
                            transaction_state.remove_completed_background_tokens();
                        }

                        inner.active_transaction_keepalives.remove(&opaque_record_key);
                    }

                    // Move in peek lock and drop it here explictly
                    drop(peek_lock);
                });
            }
        }

        Ok(())
    }
}
