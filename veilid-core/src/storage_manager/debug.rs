use super::*;

impl StorageManager {
    pub fn debug_local_records(&self) -> String {
        let Ok(local_record_store) = self.get_local_record_store() else {
            return "not initialized".to_owned();
        };
        local_record_store.debug_records()
    }
    pub fn debug_remote_records(&self) -> String {
        let Ok(remote_record_store) = self.get_remote_record_store() else {
            return "not initialized".to_owned();
        };
        remote_record_store.debug_records()
    }
    pub fn debug_opened_records(&self) -> String {
        let inner = self.inner.lock();
        let mut out = "[\n".to_owned();
        for (k, v) in &inner.opened_records {
            let writer = if let Some(w) = v.writer() {
                w.to_string()
            } else {
                "".to_owned()
            };
            let encryption_key = if let Some(e) = v.encryption_key() {
                format!(":{}", e)
            } else {
                "".to_owned()
            };
            out += &format!("  {}{} {}\n", k, encryption_key, writer);
        }
        format!("{}]\n", out)
    }
    pub fn debug_watched_records(&self) -> String {
        let inner = self.inner.lock();
        inner.outbound_watch_manager.to_string()
    }
    pub fn debug_transactions(&self) -> String {
        let inner = self.inner.lock();
        inner.outbound_transaction_manager.to_string()
    }

    pub fn debug_offline_records(&self) -> String {
        let inner = self.inner.lock();
        let Some(local_record_store) = inner.local_record_store.clone() else {
            return "not initialized".to_owned();
        };

        let mut out = "[\n".to_owned();
        for (k, v) in &inner.offline_subkey_writes {
            let record_info = local_record_store
                .peek_record(k, |r| format!("{} nodes", r.detail().nodes.len()))
                .unwrap_or("Not found".to_owned());

            out += &format!("  {}:{:?}, {}\n", k, v, record_info);
        }
        format!("{}]\n", out)
    }

    pub async fn purge_local_records(&self, reclaim: Option<u64>) -> String {
        let local_record_store = {
            let inner = self.inner.lock();
            let Some(local_record_store) = inner.local_record_store.clone() else {
                return "not initialized".to_owned();
            };
            if !inner.opened_records.is_empty() {
                return "records still opened".to_owned();
            }

            local_record_store
        };
        let reclaimed_space = local_record_store
            .reclaim_space(reclaim.unwrap_or(u64::MAX))
            .await;
        let record_locks = self
            .record_lock_table
            .lock_records(
                reclaimed_space.dead_records,
                StorageManagerRecordLockPurpose::Delete,
            )
            .await;

        if let Err(e) = self.cleanup_records_locked(&record_locks).await {
            veilid_log!(self error "Error cleaning up records in local purge: {}", e);
        }

        format!(
            "Local records purged: purged {} bytes, now {} bytes total",
            reclaimed_space.reclaimed, reclaimed_space.total
        )
    }

    pub async fn purge_remote_records(&self, reclaim: Option<u64>) -> String {
        let remote_record_store = {
            let inner = self.inner.lock();
            let Some(remote_record_store) = inner.remote_record_store.clone() else {
                return "not initialized".to_owned();
            };
            if !inner.opened_records.is_empty() {
                return "records still opened".to_owned();
            }
            remote_record_store
        };
        let reclaimed_space = remote_record_store
            .reclaim_space(reclaim.unwrap_or(u64::MAX))
            .await;
        format!(
            "Remote records purged: reclaimed {} bytes, now {} bytes total",
            reclaimed_space.reclaimed, reclaimed_space.total
        )
    }

    pub async fn debug_local_record_subkey_info(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
    ) -> String {
        let local_record_store = {
            let inner = self.inner.lock();
            let Some(local_record_store) = inner.local_record_store.clone() else {
                return "not initialized".to_owned();
            };
            local_record_store
        };
        let opaque_record_key = record_key.opaque();
        local_record_store
            .debug_record_subkey_info(&opaque_record_key, subkey)
            .await
    }
    pub async fn debug_remote_record_subkey_info(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
    ) -> String {
        let remote_record_store = {
            let inner = self.inner.lock();
            let Some(remote_record_store) = inner.remote_record_store.clone() else {
                return "not initialized".to_owned();
            };
            remote_record_store
        };
        let opaque_record_key = record_key.opaque();
        remote_record_store
            .debug_record_subkey_info(&opaque_record_key, subkey)
            .await
    }
    pub fn debug_local_record_info(&self, record_key: RecordKey) -> String {
        let opaque_record_key = record_key.opaque();

        let (local_record_store, opened_debug) = {
            let inner = self.inner.lock();
            let Some(local_record_store) = inner.local_record_store.clone() else {
                return "not initialized".to_owned();
            };
            let opened_debug = if let Some(o) = inner.opened_records.get(&opaque_record_key) {
                format!("Opened Record: {:#?}\n", o)
            } else {
                "".to_owned()
            };

            (local_record_store, opened_debug)
        };
        let local_debug = local_record_store.debug_record_info(&opaque_record_key);

        format!("{}\n{}", local_debug, opened_debug)
    }

    pub fn debug_remote_record_info(&self, record_key: RecordKey) -> String {
        let remote_record_store = {
            let inner = self.inner.lock();
            let Some(remote_record_store) = inner.remote_record_store.clone() else {
                return "not initialized".to_owned();
            };
            remote_record_store
        };
        let opaque_record_key = record_key.opaque();
        remote_record_store.debug_record_info(&opaque_record_key)
    }
}
