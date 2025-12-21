use futures_util::StreamExt as _;

use super::*;

/// Maximum number of records per transaction
const MAX_RECORDS_PER_TRANSACTION: usize = 32;

impl_veilid_log_facility!("stor");

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutboundTransactionHandle {
    keys: Arc<Vec<OpaqueRecordKey>>,
}

impl OutboundTransactionHandle {
    pub fn new(keys: Vec<OpaqueRecordKey>) -> Self {
        Self {
            keys: Arc::new(keys),
        }
    }

    pub fn keys(&self) -> &[OpaqueRecordKey] {
        self.keys.as_ref()
    }
}

impl fmt::Display for OutboundTransactionHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let othstr = self
            .keys
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "[{}]", othstr)
    }
}

impl StorageManager {
    /// Create a new outbound transaction over a set of records
    /// If an existing transaction exists over these records
    /// or a transaction can not be performed at this time, this will fail.
    /// Returns a transaction handle if the transaction was created
    /// Returns Err(VeilidAPIError::TryAgain) if the transaction could not be created
    #[instrument(level = "trace", target = "stor", skip(self), ret)]
    pub async fn begin_transaction(
        &self,
        record_keys: Vec<RecordKey>,
        options: Option<TransactDHTRecordsOptions>,
    ) -> VeilidAPIResult<OutboundTransactionHandle> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        // Early rejection if no records are being transacted over
        if record_keys.is_empty() {
            apibail_missing_argument!(
                "begin_transaction requires one or more records",
                "record_keys"
            );
        }

        // Enforce record limit
        if record_keys.len() > MAX_RECORDS_PER_TRANSACTION {
            apibail_invalid_argument!(
                format!(
                    "begin_transaction has more than {} records",
                    MAX_RECORDS_PER_TRANSACTION
                ),
                "record_keys",
                record_keys.len()
            );
        }

        // Early rejection if there are duplicate records
        if record_keys.has_duplicates() {
            apibail_missing_argument!(
                "transaction can not have duplicate record keys",
                "record_keys"
            );
        }

        let records_lock = self
            .record_lock_table
            .lock_records(
                record_keys.iter().map(|x| x.opaque()).collect(),
                StorageManagerRecordLockPurpose::TransactBegin,
            )
            .await;

        // Early rejection if dht is not online
        if !self.dht_is_online() {
            apibail_try_again!("dht is not online");
        }

        // Resolve options
        let options = options.unwrap_or_default();
        let required_strict_consensus_count = self.config().network.dht.set_value_count as usize;

        // Get opened records and construct record states
        let (transaction_handle, begin_params_list) = {
            let mut inner = self.inner.lock();

            let mut record_params = vec![];
            for record_key in record_keys {
                let opaque_record_key = record_key.opaque();
                let Some(opened_record) = inner.opened_records.get(&opaque_record_key) else {
                    apibail_generic!("record key not open: {}", opaque_record_key);
                };
                if record_key.encryption_key().map(|x| x.value()) != opened_record.encryption_key()
                {
                    apibail_generic!(
                        "record encryption key does not match opened record encryption key: {}",
                        opaque_record_key
                    );
                }

                // Get signing keypair for this transaction
                let signing_keypair = opened_record
                    .writer()
                    .cloned()
                    .or_else(|| options.default_signing_keypair.clone())
                    .unwrap_or_else(|| {
                        self.anonymous_signing_keys
                            .get(opaque_record_key.kind())
                            .unwrap()
                    });

                // Get safety selection for this record
                let safety_selection = opened_record.safety_selection();

                // Add parameters for this record
                record_params.push(OutboundTransactionRecordParams {
                    record_key,
                    signing_keypair,
                    required_strict_consensus_count,
                    safety_selection,
                });
            }

            // Obtain the outbound transaction manager
            let otm = &mut inner.outbound_transaction_manager;

            // Create a new transaction if possible
            let transaction_handle = otm.new_transaction(record_params)?;

            // Get parameters for beginning a transaction
            let begin_params_list =
                match otm.prepare_transact_begin_params(transaction_handle.clone()) {
                    Ok(v) => v,
                    Err(e) => {
                        veilid_log!(self debug "error in prepare_transact_begin_params: {}", e);

                        // Drop the transaction and ignore the result because there can't be any background tokens yet
                        let _ = otm.drop_transaction(transaction_handle);

                        return Err(e);
                    }
                };

            (transaction_handle, begin_params_list)
        };

        self.rollback_guard_locked(&records_lock, transaction_handle.clone(), async {
            // Send outbound begin transactions on all records over the network
            let mut unord = FuturesUnordered::new();
            for begin_params in begin_params_list {
                let fut = self.outbound_transact_begin(begin_params).measure_debug(
                    TimestampDuration::new_secs(5),
                    veilid_log_dbg!(
                        self,
                        "StorageManager::begin_transaction outbound_transact_begin"
                    ),
                );
                unord.push(fut);
            }
            let mut results = vec![];
            let mut opt_begin_error = None;
            while let Some(res) = unord.next().await {
                match res {
                    Ok(v) => {
                        //
                        results.push(v);
                    }
                    Err(e) => {
                        veilid_log!(self debug "error in outbound_transact_begin: {}", e);
                        if opt_begin_error.is_none() {
                            opt_begin_error = Some(e);
                        }
                    }
                }
            }

            // Snapshot local valuedata for transaction
            let local_record_store = self.get_local_record_store()?;

            let local_snapshots = {
                let mut local_snapshot_locks = vec![];
                for opaque_record_key in transaction_handle.keys() {
                    local_snapshot_locks.push(
                        local_record_store
                            .prepare_snapshot_lock(opaque_record_key.clone())
                            .await,
                    );
                }

                let mut local_snapshots = vec![];
                for local_snapshot_lock in local_snapshot_locks {
                    if let Some(local_snapshot) = local_record_store
                        .snapshot_record_locked(&local_snapshot_lock)
                        .await?
                    {
                        local_snapshots.push((local_snapshot_lock.record(), local_snapshot));
                    }
                }
                local_snapshots
            };

            {
                let mut inner = self.inner.lock();
                let transaction_state = inner
                    .outbound_transaction_manager
                    .get_transaction_state_mut(&transaction_handle)?;

                for (opaque_record_key, local_snapshot) in local_snapshots {
                    let record_state = transaction_state
                        .get_record_state_mut(&opaque_record_key)
                        .ok_or_else(|| {
                        VeilidAPIError::internal(format!(
                            "missing record state: {}",
                            opaque_record_key
                        ))
                    })?;

                    record_state.set_local_snapshot(local_snapshot);
                }
            }

            // Keep the list of nodes that returned a value for later reference
            for result in &results {
                let subkey_count = result.descriptor.schema()?.subkey_count();
                if result.seqs.len() != subkey_count && !result.fanout_result.value_nodes.is_empty()
                {
                    apibail_internal!(
                        "seqs returned does not match subkey count: {} != {}: {:?}",
                        result.seqs.len(),
                        subkey_count,
                        result
                    );
                }
                let max_subkey = result.descriptor.schema()?.max_subkey();

                let existed = self.process_fanout_results(
                    result.opaque_record_key.clone(),
                    core::iter::once((
                        ValueSubkeyRangeSet::single_range(0, max_subkey),
                        result.fanout_result.clone(),
                    )),
                    false,
                    self.config().network.dht.consensus_width as usize,
                )?;
                if !existed {
                    apibail_internal!(
                        "Record went missing during transaction despite lock: {}",
                        result.opaque_record_key
                    );
                }
            }

            if let Err(e) = self
                .inner
                .lock()
                .outbound_transaction_manager
                .record_transact_begin_results(transaction_handle.clone(), results)
            {
                veilid_log!(self debug "error in record_transact_begin_results: {}", e);
                if opt_begin_error.is_none() {
                    opt_begin_error = Some(e);
                }
            }

            // Rollback if any errors happened
            if let Some(begin_error) = opt_begin_error {
                return Err(begin_error);
            }

            // Otherwise return handle
            Ok(transaction_handle)
        })
        .await
    }

    /// Finalize a transaction over a set of records
    /// If an existing transaction does not exist over these records
    /// or a transaction can not be performed at this time, this will fail.
    /// Returns Err(VeilidAPIError::TryAgain) if the transaction could not be finalized at this time
    /// Returns Err(_) if the transaction finalize failed and resulted in rollback or drop
    #[instrument(level = "trace", target = "stor", skip(self))]
    pub async fn end_and_commit_transaction(
        &self,
        transaction_handle: OutboundTransactionHandle,
    ) -> VeilidAPIResult<()> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        // Early rejection if dht is not online
        if !self.dht_is_online() {
            apibail_try_again!("dht is not online");
        }

        let records_lock = self
            .record_lock_table
            .lock_records(
                transaction_handle.keys().to_vec(),
                StorageManagerRecordLockPurpose::TransactEndAndCommit,
            )
            .await;

        self.end_transaction_locked(&records_lock, transaction_handle.clone())
            .await?;

        self.commit_transaction_locked(&records_lock, transaction_handle.clone())
            .await?;

        // If we get here, it's time to push everything
        // to the local record store and drop the transaction
        self.flush_committed_transaction_locked(&records_lock, transaction_handle)
            .await;

        Ok(())
    }

    /// End a transaction over a set of records
    /// If an existing transaction does not exist over these records
    /// or a transaction can not be performed at this time, this will fail.
    /// Returns Err(VeilidAPIError::TryAgain) if the transaction could not be ended at this time
    /// Returns Err(_) if the transaction end failed and resulted in rollback or drop
    #[instrument(level = "trace", target = "stor", skip(self, records_lock))]
    pub(super) async fn end_transaction_locked(
        &self,
        records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
    ) -> VeilidAPIResult<()> {
        Box::pin(
            self.rollback_guard_locked(records_lock, transaction_handle.clone(), async move {
                let command_params_list = {
                    let mut inner = self.inner.lock();

                    // Obtain the outbound transaction manager
                    let otm = &mut inner.outbound_transaction_manager;

                    // Prepare for rollback
                    otm.prepare_transact_end_params(transaction_handle.clone())
                        .inspect_err(|e| {
                            veilid_log!(self debug "error in prepare_transact_end_params: {}", e);
                        })?
                };

                // End transactions on all records
                let mut unord = FuturesUnordered::new();
                for command_params in command_params_list {
                    let fut = self
                        .outbound_transact_command(command_params)
                        .measure_debug(
                            TimestampDuration::new_secs(5),
                            veilid_log_dbg!(
                                self,
                                "StorageManager::end_transaction_locked outbound_transact_command"
                            ),
                        );
                    unord.push(fut);
                }
                let mut results = vec![];
                let mut opt_end_error = None;
                while let Some(res) = unord.next().await {
                    match res {
                        Ok(v) => {
                            //
                            results.push(v);
                        }
                        Err(e) => {
                            veilid_log!(self debug "error in end transaction: {}", e);
                            if opt_end_error.is_none() {
                                opt_end_error = Some(e);
                            }
                        }
                    }
                }

                // Store end results
                {
                    let mut inner = self.inner.lock();
                    let otm = &mut inner.outbound_transaction_manager;
                    if let Err(e) =
                        otm.record_transact_end_results(transaction_handle.clone(), results)
                    {
                        veilid_log!(self debug "Recording end transaction failed: {}", e);
                        if opt_end_error.is_none() {
                            opt_end_error = Some(e);
                        }
                    }
                };

                // Rollback if any errors happened
                if let Some(end_error) = opt_end_error {
                    return Err(end_error);
                }

                Ok(())
            }),
        )
        .await
    }

    /// Commit a transaction over a set of records
    /// If an existing transaction does not exist over these records
    /// or a transaction can not be performed at this time, this will fail.
    /// Returns Err(VeilidAPIError::TryAgain) if the transaction could not be committed at this time
    /// Returns Err(_) if the transaction commit failed and resulted in rollback or drop
    #[instrument(level = "trace", target = "stor", skip(self, records_lock))]
    pub(super) async fn commit_transaction_locked(
        &self,
        records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
    ) -> VeilidAPIResult<()> {
        Box::pin(
            self.rollback_guard_locked(records_lock, transaction_handle.clone(), async move {
                let command_params_list = {
                    let mut inner = self.inner.lock();

                    // Obtain the outbound transaction manager
                    let otm = &mut inner.outbound_transaction_manager;

                    // Prepare for commit
                    otm.prepare_transact_commit_params(transaction_handle.clone())
                    .inspect_err(|e| {
                        veilid_log!(self debug "error in prepare_transact_commit_params: {}", e);
                    })?
                };

                // Commit transactions on all records
                let mut unord = FuturesUnordered::new();
                for command_params in command_params_list {
                    let fut = self
                        .outbound_transact_command(command_params)
                        .measure_debug(
                            TimestampDuration::new_secs(5),
                            veilid_log_dbg!(
                            self,
                            "StorageManager::commit_transaction_locked outbound_transact_command"
                        ),
                        );
                    unord.push(fut);
                }
                let mut results = vec![];
                let mut opt_commit_error = None;
                while let Some(res) = unord.next().await {
                    match res {
                        Ok(v) => {
                            //
                            results.push(v);
                        }
                        Err(e) => {
                            veilid_log!(self debug "Commit transaction failed: {}", e);

                            if opt_commit_error.is_none() {
                                opt_commit_error = Some(e);
                            }
                        }
                    }
                }

                // Store commit results
                {
                    let mut inner = self.inner.lock();
                    if let Err(e) = inner
                        .outbound_transaction_manager
                        .record_transact_commit_results(transaction_handle.clone(), results)
                    {
                        veilid_log!(self debug "Recording commit transaction failed: {}", e);

                        if opt_commit_error.is_none() {
                            opt_commit_error = Some(e);
                        }
                    }

                    if let Some(err) = opt_commit_error {
                        return Err(err);
                    }
                }
                Ok(())
            }),
        )
        .await
    }

    /// Removes the transaction from the transaction manager
    /// and flushes its contents to the storage manager
    #[instrument(level = "trace", target = "dht", skip(self, records_lock))]
    pub(super) async fn flush_committed_transaction_locked(
        &self,
        records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
    ) {
        let (keys_and_subkeys, background_tokens) = {
            let mut inner = self.inner.lock();

            let Some(transaction_state) = inner
                .outbound_transaction_manager
                .drop_transaction(transaction_handle.clone())
            else {
                veilid_log!(self error "missing transaction in flush: {}", transaction_handle);
                return;
            };

            let mut keys_and_subkeys = vec![];
            for record_state in transaction_state.get_record_states() {
                let opaque_record_key = record_state.record_key().opaque();
                let local_commit_results = match record_state.local_commit_results() {
                    Ok(v) => v,
                    Err(e) => {
                        veilid_log!(self error "failed to get local commit results for transaction {}: {}", transaction_handle, e);
                        return;
                    }
                };

                keys_and_subkeys.push((opaque_record_key, local_commit_results));
            }

            (keys_and_subkeys, transaction_state.into_background_tokens())
        };

        // Wait for all background operations to finish on the transaction
        Self::wait_for_background_tokens(background_tokens).await;

        // Record the set values locally since they were successfully set online
        if let Err(e) = self
            .handle_set_local_values_with_multiple_records_lock(records_lock, keys_and_subkeys)
            .await
        {
            veilid_log!(self error "failed to set local values with commit results for transaction {}: {}", transaction_handle, e);
        }
    }

    /// Roll back a transaction
    /// If the transaction no longer exists, this does nothing.
    /// If an error is returned, the transaction is left in a failed state and can either
    /// * be dropped/ignored and the remote transaction will time out
    /// * another rollback attempt can be made, which may result in a more polite termination of the remote transaction
    #[instrument(level = "trace", target = "dht", skip(self))]
    pub async fn rollback_transaction(
        &self,
        transaction_handle: OutboundTransactionHandle,
    ) -> VeilidAPIResult<()> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };
        let records_lock = self
            .record_lock_table
            .lock_records(
                transaction_handle.keys().to_vec(),
                StorageManagerRecordLockPurpose::TransactRollback,
            )
            .await;

        // Early exit if transaction is already gone
        if !self
            .inner
            .lock()
            .outbound_transaction_manager
            .transaction_exists(&transaction_handle)
        {
            return Ok(());
        }

        // Early rejection if dht is not online
        if !self.dht_is_online() {
            apibail_try_again!("dht is not online");
        }

        // Send all rollbacks to the network
        self.rollback_transaction_locked(&records_lock, transaction_handle.clone())
            .await?;

        // Transaction is done successfully, drop it and wait for background tasks to complete if any
        self.drop_transaction_and_wait(transaction_handle).await;

        Ok(())
    }

    #[instrument(level = "trace", target = "dht", skip(self, _records_lock))]
    pub(super) async fn rollback_transaction_locked(
        &self,
        _records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
    ) -> VeilidAPIResult<()> {
        let command_params_list = {
            let mut inner = self.inner.lock();

            // Obtain the outbound transaction manager
            let otm = &mut inner.outbound_transaction_manager;

            // Prepare for rollback
            otm.prepare_rollback_transact_value_params(transaction_handle.clone(), None)
                .inspect_err(|e| {
                    veilid_log!(self debug "error in prepare_rollback_transact_value_params: {}", e);
                })?
        };

        // Rollback transactions on all records
        let mut unord = FuturesUnordered::new();
        for command_params in command_params_list {
            let fut = self
                .outbound_transact_command(command_params)
                .measure_debug(
                    TimestampDuration::new_secs(5),
                    veilid_log_dbg!(
                        self,
                        "StorageManager::rollback_transaction_locked outbound_transact_command"
                    ),
                );
            unord.push(fut);
        }
        let mut results = vec![];
        let mut opt_rollback_error = None;
        while let Some(res) = unord.next().await {
            match res {
                Ok(v) => {
                    //
                    results.push(v);
                }
                Err(e) => {
                    if opt_rollback_error.is_none() {
                        opt_rollback_error = Some(e);
                    }
                }
            }
        }

        // Store rollback results
        {
            let mut inner = self.inner.lock();
            let otm = &mut inner.outbound_transaction_manager;
            if let Err(e) =
                otm.record_transact_rollback_results(transaction_handle.clone(), results)
            {
                if opt_rollback_error.is_none() {
                    opt_rollback_error = Some(e);
                }
            }
        }

        if let Some(rberr) = opt_rollback_error {
            return Err(rberr);
        }

        Ok(())
    }

    /// Get a value within a transaction
    /// Does not use fanout
    #[instrument(level = "trace", target = "dht", skip(self), ret)]
    pub async fn transaction_get(
        &self,
        transaction_handle: OutboundTransactionHandle,
        record_key: RecordKey,
        subkey: ValueSubkey,
    ) -> VeilidAPIResult<Option<ValueData>> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        let _subkey_lock = self.record_lock_table.lock_subkey(
            record_key.opaque(),
            subkey,
            StorageManagerSubkeyLockPurpose::TransactGet,
        );

        // Early rejection if dht is not online
        if !self.dht_is_online() {
            apibail_try_again!("dht is not online");
        }

        let (concurrency_semaphore, command_params) = {
            let opaque_record_key = record_key.opaque();

            let mut inner = self.inner.lock();
            let otm = &mut inner.outbound_transaction_manager;
            let concurrency_semaphore = otm
                .get_transaction_state(&transaction_handle)?
                .get_operation_concurrency_semaphore();

            // Prepare for get value
            let command_params = otm
                .prepare_transact_get_params(transaction_handle.clone(), &opaque_record_key, subkey)
                .inspect_err(|e| {
                    veilid_log!(self debug "error in prepare_transact_get_params: {}", e);
                })?;
            (concurrency_semaphore, command_params)
        };

        // Wait for concurrency semaphore
        let sem = concurrency_semaphore.acquire().await;

        // Send all get commands
        let result = self
            .outbound_transact_command(command_params)
            .measure_debug(
                TimestampDuration::new_secs(5),
                veilid_log_dbg!(
                    self,
                    "StorageManager::transaction_get outbound_transact_command"
                ),
            )
            .await
            .inspect_err(|e| {
                veilid_log!(self debug "Transaction get failed: {}", e);
            })?;

        // Done with network access, release the semaphore
        drop(sem);

        let subkey_get_result = {
            let mut inner = self.inner.lock();
            let otm = &mut inner.outbound_transaction_manager;
            otm.record_transact_get_result(transaction_handle.clone(), result)
                .inspect_err(|e| {
                    veilid_log!(self debug "Recording get transaction failed: {}", e);
                })?;

            // Return newest value
            let outbound_transaction_state = otm
                .get_transaction_state(&transaction_handle)
                .inspect_err(|e| {
                    veilid_log!(self debug "Missing transaction state: {}", e);
                })?;
            let Some(record_state) =
                outbound_transaction_state.get_record_state(&record_key.opaque())
            else {
                apibail_internal!("missing record in get: {}", record_key.opaque());
            };
            record_state.current_subkey_get_result(subkey)?
        };
        let Some(get_signed_value_data) = subkey_get_result.opt_value else {
            // No value
            return Ok(None);
        };
        let get_value_data =
            self.maybe_decrypt_value_data(&record_key, get_signed_value_data.value_data())?;

        // Return the value we got
        Ok(Some(get_value_data))
    }

    /// Set a value within a transaction
    /// Does not use fanout
    #[instrument(level = "trace", target = "dht", skip(self, data), fields(data.len = data.len()), ret)]
    pub async fn transaction_set(
        &self,
        transaction_handle: OutboundTransactionHandle,
        record_key: RecordKey,
        subkey: ValueSubkey,
        data: Vec<u8>,
        options: Option<DHTTransactionSetValueOptions>,
    ) -> VeilidAPIResult<Option<ValueData>> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        let _subkey_lock = self.record_lock_table.lock_subkey(
            record_key.opaque(),
            subkey,
            StorageManagerSubkeyLockPurpose::TransactSet,
        );

        let opaque_record_key = record_key.opaque();

        // Early rejection if dht is not online
        if !self.dht_is_online() {
            apibail_try_again!("dht is not online");
        }

        let (concurrency_semaphore, command_params) = {
            let inner = &mut *self.inner.lock();
            let otm = &mut inner.outbound_transaction_manager;
            let concurrency_semaphore = otm
                .get_transaction_state(&transaction_handle)?
                .get_operation_concurrency_semaphore();

            // Get last known value for this subkey from the transaction
            let last_get_result = {
                let outbound_transaction_state = otm.get_transaction_state(&transaction_handle)?;
                let record_state = outbound_transaction_state
                    .get_record_state(&opaque_record_key)
                    .ok_or_else(|| VeilidAPIError::internal("missing record state"))?;

                record_state.current_subkey_get_result(subkey)?
            };

            // Use the specified writer, or if not specified, the default writer when the record was opened
            let opt_writer = {
                let Some(opened_record) = inner.opened_records.get(&opaque_record_key) else {
                    apibail_generic!("record not open");
                };
                opened_record.writer().cloned()
            };
            let opt_writer = options
                .as_ref()
                .and_then(|o| o.writer.clone())
                .or(opt_writer);

            // If we don't have a writer then we can't write
            let Some(writer) = opt_writer else {
                apibail_generic!("value is not writable");
            };

            // Make signed value data (encrypted) and value data (unencrypted) and get descriptor for this value
            let (signed_value_data, _, _) =
                self.prepare_set_value_data(&record_key, subkey, data, &writer, last_get_result)?;

            // Prepare for set value
            let command_params = otm
                .prepare_transact_set_params(
                    transaction_handle.clone(),
                    &opaque_record_key,
                    subkey,
                    signed_value_data.clone(),
                )
                .inspect_err(|e| {
                    veilid_log!(self debug "error in prepare_transact_set_params: {}", e);
                })?;

            (concurrency_semaphore, command_params)
        };

        // Wait for concurrency semaphore
        let sem = concurrency_semaphore.acquire().await;

        // Send all set commands
        let result = self
            .outbound_transact_command(command_params)
            .measure_debug(
                TimestampDuration::new_secs(5),
                veilid_log_dbg!(
                    self,
                    "StorageManager::transaction_set outbound_transact_command"
                ),
            )
            .await
            .inspect_err(|e| {
                veilid_log!(self debug "Transaction set failed: {}", e);
            })?;

        // Done with network access, release the semaphore
        drop(sem);

        let opt_current_signed_value_data = {
            let mut inner = self.inner.lock();
            let otm = &mut inner.outbound_transaction_manager;
            otm.record_transact_set_result(transaction_handle.clone(), result)
                .inspect_err(|e| {
                    veilid_log!(self debug "Recording set transaction failed: {}", e);
                })?;

            // Return newer value if it is not what we set
            let outbound_transaction_state = otm
                .get_transaction_state(&transaction_handle)
                .inspect_err(|e| {
                    veilid_log!(self debug "Missing transaction state: {}", e);
                })?;

            let record_state = outbound_transaction_state
                .get_record_state(&opaque_record_key)
                .ok_or_else(|| VeilidAPIError::internal("missing record state"))?;

            // If there is an updated value, it means the set succeeded
            // If the set found a newer value online then this gets cleared for the subkey
            if let Some(updated_consensus) = record_state.updated_consensus().get(subkey) {
                // There is an updated value after we did the set

                // Ensure the updated consensus meets the strict consensus requirement
                if updated_consensus.strict_consensus_count
                    < record_state.required_strict_consensus_count()
                {
                    // Otherwise, ask the app to try the set again to continue to attempt consensus
                    apibail_try_again!("set did not reach consensus");
                }

                // Return that the set updated with consensus successfully
                return Ok(None);
            };

            // If the set found a newer value it would be recorded in the current consensus
            // unless an error condition was hit, in which case we should have failed out with an error
            let Some(current_subkey_consensus) = record_state.current_consensus().get(subkey)
            else {
                apibail_internal!(
                    "record subkey {} should have a current consensus: {}",
                    subkey,
                    record_key.opaque()
                );
            };

            // Return current subkey consensus value data
            current_subkey_consensus.opt_value.clone()
        };

        let Some(current_signed_value_data) = opt_current_signed_value_data else {
            apibail_internal!(
                "record subkey {} consensus value should not be missing: {}",
                subkey,
                record_key.opaque()
            );
        };
        let current_value_data =
            self.maybe_decrypt_value_data(&record_key, current_signed_value_data.value_data())?;

        // Return that a newer or different value was found online
        Ok(Some(current_value_data))
    }

    /// Inspect a record within a transaction, does not perform any network
    /// activity, as the transaction state keeps all of the required information
    /// after the begin.
    #[instrument(level = "trace", target = "dht", skip(self), ret)]
    pub async fn transaction_inspect(
        &self,
        transaction_handle: OutboundTransactionHandle,
        record_key: RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        scope: DHTReportScope,
    ) -> VeilidAPIResult<DHTRecordReport> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        let mut inner = self.inner.lock();
        inner.outbound_transaction_manager.get_record_report(
            transaction_handle,
            &record_key.opaque(),
            subkeys,
            scope,
        )
    }

    /// Background rollback function used to remove nodes from a transactioni
    /// and speculatively issue rollback RPCs to them to help them release their server
    /// side transactions early. Runs detached in the background as we never care about
    /// the result.
    pub(super) fn partial_drop_and_background_rollback_locked(
        &self,
        _records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
        per_record_node_xids_to_drop: PerRecordNodeTransactionIds,
        per_record_node_xids_to_rollback: PerRecordNodeTransactionIds,
    ) -> VeilidAPIResult<()> {
        let stop_source = StopSource::new();

        let command_params_list = {
            let mut inner = self.inner.lock();

            // Obtain the outbound transaction manager
            let otm = &mut inner.outbound_transaction_manager;

            // Prepare all rollbacks -first-
            let command_params_list = otm.prepare_rollback_transact_value_params(
                transaction_handle.clone(),
                Some(per_record_node_xids_to_rollback),
            )
            .inspect_err(|e| {
                veilid_log!(self debug "error in prepare_rollback_transact_value_params: {}", e);
            })?;

            // Then process all node xid drops -second-
            let state = inner
                .outbound_transaction_manager
                .get_transaction_state_mut(&transaction_handle)?;

            // Remove drops from the transaction -second-
            for (opaque_record_key, node_xids_to_drop) in per_record_node_xids_to_drop {
                let Some(record_state) = state.get_record_state_mut(&opaque_record_key) else {
                    veilid_log!(self debug "Missing record state for {} in transaction in background drop", opaque_record_key);
                    continue;
                };

                record_state.remove_node_transactions(&node_xids_to_drop);
            }

            // Add the background task stop token to this transaction's drop wait list
            let stop_token = stop_source.token();
            state.add_background_token(stop_token);

            command_params_list
        };

        // Process background rollbacks -third-
        let registry = self.registry();
        let background_rollback_fut = async move {
            let this = registry.storage_manager();

            // Rollback transactions on all records
            let mut unord = FuturesUnordered::new();
            for command_params in command_params_list {
                let fut = this
                    .outbound_transact_command(command_params)
                    .measure_debug(
                        TimestampDuration::new_secs(5),
                        veilid_log_dbg!(
                            this,
                            "StorageManager::partial_drop_and_background_rollback_locked outbound_transact_command"
                        ),
                    );
                unord.push(fut);
            }
            while let Some(res) = unord.next().await {
                match res {
                    Ok(result) => {
                        let mut command_node_xids = result.get_command_node_xids();
                        for pnr in result.per_node_results {
                            if !command_node_xids.remove(&pnr.node_transaction_id) {
                                veilid_log!(this debug
                                    "node transaction has multiple results: {} pnr={:?}",
                                    result.params.opaque_record_key,
                                    pnr
                                );
                            }
                        }

                        // Any commands that did not return a result the background rollback
                        if !command_node_xids.is_empty() {
                            veilid_log!(this debug "Partial rollback of {} failed for: {:?}", transaction_handle, command_node_xids);
                        }
                    }
                    Err(e) => {
                        veilid_log!(this debug "Error in partial_drop_and_background_rollback_locked: {}", e);
                    }
                }
            }

            // Move the stop source in here and drop it when we're done
            {
                let mut inner = this.inner.lock();
                drop(stop_source);
                if let Ok(transaction_state) = inner
                    .outbound_transaction_manager
                    .get_transaction_state_mut(&transaction_handle)
                {
                    transaction_state.remove_completed_background_tokens();
                }
            }
        };

        // Attach this stop token to the transaction
        self.background_operation_processor
            .add_future(background_rollback_fut);

        Ok(())
    }

    /// Guard function used to ensure that errors on whole-transaction operations cause rollback attempts
    /// Also validates that the state is the same for all records in the transaction and attempts to
    /// reconcile node states that are different.
    /// For example, if a single node ends up in an 'End' state while other nodes end up in 'Rollback'
    /// this routine will make a best-effort attempt to rollback the 'End' state node.
    pub(super) async fn rollback_guard_locked<V, F: Future<Output = VeilidAPIResult<V>>>(
        &self,
        records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
        future: F,
    ) -> VeilidAPIResult<V> {
        let res = future.await;

        let mut opt_background_tokens = None;

        let out = match res {
            Ok(v) => {
                // If results are okay, process the stage consensus operations
                self.rollback_guard_locked_success(
                    records_lock,
                    transaction_handle,
                    v,
                    &mut opt_background_tokens,
                )
            }
            Err(e) => {
                // If there was an error, we always want to roll back unless the transaction has completed
                veilid_log!(self debug target: "network_result", "Rolling back due to error: {:?}: {}", transaction_handle, e);

                // Roll back everything
                if let Err(rbe) = self
                    .rollback_transaction_locked(records_lock, transaction_handle.clone())
                    .await
                {
                    veilid_log!(self debug "Error in roll back transaction: {}", rbe);
                }

                // Drop the transaction and wait for background tasks to complete if any
                self.drop_transaction_and_wait(transaction_handle).await;

                return Err(e);
            }
        };

        // If we have to wait for some background operations, do that before returning
        if let Some(background_tokens) = opt_background_tokens {
            Self::wait_for_background_tokens(background_tokens).await;
        }

        out
    }

    // Process stage consensus operations
    // Returns either the value or an error if consensus operations could not be performed
    // Also returns a list of background tokens to wait for through the mutable reference parameter
    fn rollback_guard_locked_success<V>(
        &self,
        records_lock: &StorageManagerRecordsLockGuard,
        transaction_handle: OutboundTransactionHandle,
        value: V,
        opt_background_tokens: &mut Option<Vec<StopToken>>,
    ) -> VeilidAPIResult<V> {
        let stage_consensus = {
            let mut inner = self.inner.lock();
            let res = inner
                .outbound_transaction_manager
                .get_transaction_state(&transaction_handle);
            let state = match res {
                Ok(state) => state,
                Err(e) => {
                    veilid_log!(self debug "Error getting transaction state in guard: {}", e);

                    // Drop the transaction and return background tasks to complete if any
                    *opt_background_tokens = inner
                        .outbound_transaction_manager
                        .drop_transaction(transaction_handle)
                        .map(|x| x.into_background_tokens());

                    return Err(e);
                }
            };

            let Some(stage_consensus) = state.stage_consensus() else {
                // Should not be trying to roll back something that is still in the INIT state
                apibail_internal!(
                    "no stage consensus yet for rollback guard: {}",
                    transaction_handle
                );
            };
            stage_consensus
        };

        let rollback_ids = stage_consensus.per_record_node_xids_to_rollback;
        let drop_ids = stage_consensus.per_record_node_xids_to_drop;
        if !rollback_ids.is_empty() {
            // Perform partial speculative rollback and drop from transaction
            if let Err(e) = self.partial_drop_and_background_rollback_locked(
                records_lock,
                transaction_handle.clone(),
                drop_ids,
                rollback_ids,
            ) {
                veilid_log!(self debug "Error in partial drop and roll back transaction: {}", e);

                // Drop the transaction and wait for background tasks to complete if any
                let mut inner = self.inner.lock();
                *opt_background_tokens = inner
                    .outbound_transaction_manager
                    .drop_transaction(transaction_handle)
                    .map(|x| x.into_background_tokens());

                return Err(e);
            }
        }

        Ok(value)
    }

    /// Wait for a list of background tokens to drop
    pub(super) async fn wait_for_background_tokens(background_tokens: Vec<StopToken>) {
        let mut unord = FuturesUnordered::from_iter(background_tokens.into_iter());
        while (unord.next().await).is_some() {}
    }

    /// Convenience function to drop transaction and wait for background tasks to complete
    pub(super) async fn drop_transaction_and_wait(
        &self,
        transaction_handle: OutboundTransactionHandle,
    ) {
        let opt_background_tokens = self
            .inner
            .lock()
            .outbound_transaction_manager
            .drop_transaction(transaction_handle)
            .map(|x| x.into_background_tokens());
        if let Some(background_tokens) = opt_background_tokens {
            Self::wait_for_background_tokens(background_tokens).await;
        }
    }

    /// Schedule a transaction to be dropped
    #[instrument(level = "trace", target = "dht", skip(self))]
    pub fn drop_transaction_sync(&self, transaction_handle: OutboundTransactionHandle) {
        let registry = self.registry();
        self.background_operation_processor.add_future(async move {
            let this = registry.storage_manager();

            let _records_lock = this
                .record_lock_table
                .lock_records(
                    transaction_handle.keys().to_vec(),
                    StorageManagerRecordLockPurpose::TransactDrop,
                )
                .await;

            // Drop the transaction and wait for background tasks to complete if any
            this.drop_transaction_and_wait(transaction_handle).await;
        });
    }
}
