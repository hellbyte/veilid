use super::*;

#[cfg(feature = "debug-locks")]
pub(super) static GUARD_ID: AtomicUsize = AtomicUsize::new(0);

/// Subkey lock management structure
#[derive(Debug)]
struct SubkeyLockInfo<S: LockPurpose> {
    lock_table: WeakValueHashMap<ValueSubkey, Weak<AsyncMutex<()>>>,
    purpose_table: HashMap<ValueSubkey, S>,
}

/// Record lock management structure
#[derive(Debug)]
pub(super) struct RecordLock<R: LockPurpose, S: LockPurpose> {
    record: OpaqueRecordKey,
    whole_record_lock: Arc<AsyncRwLock<()>>,
    whole_record_lock_purpose: Mutex<Option<R>>,
    subkey_lock_info: Mutex<SubkeyLockInfo<S>>,
    peek_count: Mutex<usize>,
    #[cfg(feature = "debug-locks")]
    active_guards: Arc<Mutex<HashMap<usize, backtrace::Backtrace>>>,
}

impl<R: LockPurpose, S: LockPurpose> RecordLock<R, S> {
    pub fn new(record: OpaqueRecordKey) -> Self {
        Self {
            record,
            whole_record_lock: Arc::new(AsyncRwLock::new(())),
            whole_record_lock_purpose: Mutex::new(None),
            subkey_lock_info: Mutex::new(SubkeyLockInfo {
                lock_table: WeakValueHashMap::new(),
                purpose_table: HashMap::new(),
            }),
            peek_count: Mutex::new(0),
            #[cfg(feature = "debug-locks")]
            active_guards: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record(&self) -> OpaqueRecordKey {
        self.record.clone()
    }
    pub fn get_whole_record_lock(&self) -> Arc<AsyncRwLock<()>> {
        self.whole_record_lock.clone()
    }
    pub fn purpose(&self) -> Option<R> {
        self.whole_record_lock_purpose.lock().clone()
    }
    pub fn set_record_purpose(&self, purpose: R) {
        *(self.whole_record_lock_purpose.lock()) = Some(purpose);
        self.subkey_lock_info.lock().purpose_table.clear();
    }
    pub fn get_subkey_lock(&self, subkey: ValueSubkey) -> Arc<AsyncMutex<()>> {
        let mut subkey_lock_info = self.subkey_lock_info.lock();
        subkey_lock_info.lock_table.remove_expired();
        subkey_lock_info
            .lock_table
            .entry(subkey)
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
    }
    #[expect(dead_code)]
    pub fn subkey_purpose(&self, subkey: ValueSubkey) -> Option<S> {
        self.subkey_lock_info
            .lock()
            .purpose_table
            .get(&subkey)
            .cloned()
    }
    pub fn set_subkey_purpose(&self, subkey: ValueSubkey, purpose: S) {
        *(self.whole_record_lock_purpose.lock()) = None;
        self.subkey_lock_info
            .lock()
            .purpose_table
            .insert(subkey, purpose);
    }
    pub fn get_subkey_purpose_map(&self) -> BTreeMap<S, ValueSubkeyRangeSet> {
        let mut purpose_map = BTreeMap::<S, ValueSubkeyRangeSet>::new();

        for (k, v) in self.subkey_lock_info.lock().purpose_table.iter() {
            purpose_map.entry(v.clone()).or_default().insert(*k);
        }

        purpose_map
    }

    pub fn get_peek_count(&self) -> usize {
        *self.peek_count.lock()
    }

    pub fn add_peek(&self) {
        *(self.peek_count.lock()) += 1;
    }

    #[cfg(feature = "debug-locks")]
    pub(super) fn get_active_guards(&self) -> Arc<Mutex<HashMap<usize, backtrace::Backtrace>>> {
        self.active_guards.clone()
    }

    pub(super) fn drop_record_lock_guard(&self) {
        *(self.whole_record_lock_purpose.lock()) = None;
    }

    pub(super) fn drop_subkey_lock_guard(&self, subkey: ValueSubkey) {
        self.subkey_lock_info.lock().purpose_table.remove(&subkey);
    }

    pub(super) fn drop_peek_lock_guard(&self) {
        *(self.peek_count.lock()) -= 1;
    }
}
