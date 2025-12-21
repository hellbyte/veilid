use super::*;

/// Shared record lock guard for a single record with no subkey locks
#[derive(Debug)]
pub struct PeekLockGuard<R: LockPurpose, S: LockPurpose> {
    record_lock: Arc<RecordLock<R, S>>,
    _whole_record_lock_guard: AsyncRwLockReadGuardArc<()>,
    #[cfg(feature = "debug-locks")]
    id: usize,
    #[cfg(feature = "debug-locks")]
    active_guards: Arc<Mutex<HashMap<usize, backtrace::Backtrace>>>,
}

impl<R: LockPurpose, S: LockPurpose> PeekLockGuard<R, S> {
    pub(super) fn new(
        record_lock: Arc<RecordLock<R, S>>,
        whole_record_lock_guard: AsyncRwLockReadGuardArc<()>,
    ) -> Self {
        #[cfg(feature = "debug-locks")]
        let (id, active_guards) = {
            let id = GUARD_ID.fetch_add(1, Ordering::AcqRel);
            let active_guards = record_lock.get_active_guards();
            active_guards.lock().insert(id, backtrace::Backtrace::new());
            (id, active_guards)
        };

        Self {
            record_lock,
            _whole_record_lock_guard: whole_record_lock_guard,
            #[cfg(feature = "debug-locks")]
            id,
            #[cfg(feature = "debug-locks")]
            active_guards,
        }
    }
    pub fn record(&self) -> OpaqueRecordKey {
        self.record_lock.record()
    }
}

impl<R: LockPurpose, S: LockPurpose> Drop for PeekLockGuard<R, S> {
    fn drop(&mut self) {
        #[cfg(feature = "debug-locks")]
        self.active_guards.lock().remove(&self.id);

        self.record_lock.drop_peek_lock_guard();
    }
}

impl<R: LockPurpose, S: LockPurpose> fmt::Display for PeekLockGuard<R, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Peek({})", self.record())
    }
}
