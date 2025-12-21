use super::*;

impl StorageManager {
    /// Close an opened local record
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub async fn close_record(&self, record_key: RecordKey) -> VeilidAPIResult<()> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        let opaque_record_key = record_key.opaque();
        let record_lock = self
            .record_lock_table
            .lock_record(
                opaque_record_key.clone(),
                StorageManagerRecordLockPurpose::Close,
            )
            .await;

        // Attempt to close the record
        let background_tokens = self.close_record_locked(&record_lock)?;

        Self::wait_for_background_tokens(background_tokens).await;

        Ok(())
    }

    /// Close all opened records
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub async fn close_all_records(&self) -> VeilidAPIResult<()> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        let record_locks = {
            let keys = {
                let inner = self.inner.lock();
                inner.opened_records.keys().cloned().collect::<Vec<_>>()
            };

            self.record_lock_table
                .lock_records(keys, StorageManagerRecordLockPurpose::Close)
                .await
        };

        let mut all_background_tokens = vec![];
        let mut opt_error = None;
        for record_lock_guard in record_locks.record_lock_guards() {
            let res = self.close_record_locked(record_lock_guard);
            match res {
                Ok(mut v) => {
                    all_background_tokens.append(&mut v);
                }
                Err(e) => {
                    opt_error = Some(e);
                }
            }
        }

        Self::wait_for_background_tokens(all_background_tokens).await;

        if let Some(e) = opt_error {
            return Err(e);
        }

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////

    pub(super) fn close_record_locked(
        &self,
        record_lock: &StorageManagerRecordLockGuard,
    ) -> VeilidAPIResult<Vec<StopToken>> {
        let opaque_record_key = record_lock.record();

        let local_record_store = self.get_local_record_store()?;
        if !local_record_store.contains_record(&opaque_record_key) {
            apibail_key_not_found!(opaque_record_key.clone());
        }

        let mut all_background_tokens = vec![];
        {
            let mut inner = self.inner.lock();
            if let Some(opened_record) = inner.opened_records.remove(&opaque_record_key) {
                let record_key = RecordKey::from_opaque(
                    opaque_record_key.clone(),
                    opened_record.encryption_key(),
                );

                // Set the watch to cancelled if we have one
                // Will process cancellation in the background
                inner
                    .outbound_watch_manager
                    .set_desired_watch(record_key, None);

                // Drop any transaction associated with the record
                if let Some(transaction_handle) = inner
                    .outbound_transaction_manager
                    .get_transaction_by_record(&opaque_record_key)
                {
                    if let Some(mut background_tokens) = inner
                        .outbound_transaction_manager
                        .drop_transaction(transaction_handle)
                        .map(|state| state.into_background_tokens())
                    {
                        all_background_tokens.append(&mut background_tokens);
                    }
                }
            }
        }

        Ok(all_background_tokens)
    }
}
