use super::*;

impl_veilid_log_facility!("stor");

pub(super) struct ActiveSubkeyWriteGuard {
    registry: VeilidComponentRegistry,
    done: bool,
    record_key: TypedRecordKey,
    subkey: ValueSubkey,
}

impl ActiveSubkeyWriteGuard {
    fn set_done(&mut self) {
        self.done = true;
    }
}

impl Drop for ActiveSubkeyWriteGuard {
    fn drop(&mut self) {
        if !self.done {
            let registry = &self.registry;
            veilid_log!(registry error "active subkey write finished without being marked done: {}:{}", self.record_key, self.subkey);
        }
    }
}

impl StorageManager {
    // Returns false if we were not already writing
    // Returns true if this subkey was already being written to
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub(super) fn mark_active_subkey_write_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: TypedRecordKey,
        subkey: ValueSubkey,
    ) -> Option<ActiveSubkeyWriteGuard> {
        let asw = inner.active_subkey_writes.entry(record_key).or_default();
        if asw.contains(subkey) {
            veilid_log!(self debug "already writing to this subkey: {}:{}", record_key, subkey);
            None
        } else {
            // Add to our list of active subkey writes
            asw.insert(subkey);
            Some(ActiveSubkeyWriteGuard {
                registry: self.registry(),
                done: false,
                record_key,
                subkey,
            })
        }
    }

    #[instrument(level = "trace", target = "stor", skip_all)]
    pub(super) fn unmark_active_subkey_write_inner(
        &self,
        inner: &mut StorageManagerInner,
        mut guard: ActiveSubkeyWriteGuard,
    ) {
        // Remove from active subkey writes
        let asw = inner
            .active_subkey_writes
            .get_mut(&guard.record_key)
            .unwrap();
        if !asw.remove(guard.subkey) {
            veilid_log!(self error "missing active subkey write: {}:{}", guard.record_key, guard.subkey);
        }
        if asw.is_empty() {
            inner.active_subkey_writes.remove(&guard.record_key);
        }
        guard.set_done();
    }
}
