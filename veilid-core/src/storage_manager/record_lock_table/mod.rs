mod peek_lock_guard;
mod record_lock;
mod record_lock_guard;
mod records_lock_guard;
mod subkey_lock_guard;

pub use peek_lock_guard::*;
pub use record_lock_guard::*;
pub use records_lock_guard::*;
pub use subkey_lock_guard::*;

use super::*;
use record_lock::*;
use weak_table::WeakValueHashMap;

pub trait LockPurpose:
    fmt::Debug + Clone + Eq + PartialEq + Ord + PartialOrd + core::hash::Hash
{
}

/// Types of record locks
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RecordLockKind<R: LockPurpose, S: LockPurpose> {
    Unlocked,
    RecordLocked {
        purpose: R,
    },
    SubkeyLocked {
        purpose_map: BTreeMap<S, ValueSubkeyRangeSet>,
        peek_count: usize,
    },
}

/// Table for all record locks that uses weak hash maps to auto-collect when guards drop
#[derive(Debug)]
struct RecordLockTableInner<R: LockPurpose, S: LockPurpose> {
    record_lock_table: WeakValueHashMap<OpaqueRecordKey, Weak<RecordLock<R, S>>>,
}

/// Interface to record locking mechanism
#[derive(Clone, Debug)]
pub struct RecordLockTable<R: LockPurpose, S: LockPurpose> {
    inner: Arc<Mutex<RecordLockTableInner<R, S>>>,
}

impl<R: LockPurpose, S: LockPurpose> RecordLockTable<R, S> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RecordLockTableInner::<R, S> {
                record_lock_table: WeakValueHashMap::new(),
            })),
        }
    }

    pub async fn lock_record(&self, record: OpaqueRecordKey, purpose: R) -> RecordLockGuard<R, S> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_lock = inner
                .record_lock_table
                .entry(record.clone())
                .or_insert_with(|| Arc::new(RecordLock::new(record.clone())));
            record_lock
        };

        // Wait on record write lock
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let whole_record_lock_guard = match timeout(30000, record_lock.get_whole_record_lock().write_arc()).await {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("active guards: {:#?}", record_lock.get_active_guards().lock().values().collect::<Vec<_>>());
                        panic!("lock_record deadlock");
                    }
                };
            } else {
                let whole_record_lock_guard = record_lock.get_whole_record_lock().write_arc().await;
            }
        }

        record_lock.set_record_purpose(purpose);

        RecordLockGuard::new(record_lock, whole_record_lock_guard)
    }

    pub async fn lock_records(
        &self,
        mut records: Vec<OpaqueRecordKey>,
        purpose: R,
    ) -> RecordsLockGuard<R, S> {
        // Always lock in sorted order to avoid deadlocks
        records.sort();

        // Get record locks
        let record_locks = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_locks = records
                .into_iter()
                .map(|record| {
                    inner
                        .record_lock_table
                        .entry(record.clone())
                        .or_insert_with(|| Arc::new(RecordLock::new(record.clone())))
                })
                .collect::<Vec<_>>();
            record_locks
        };

        // Wait on each record write lock to complete in order
        let mut record_lock_guards = vec![];
        for record_lock in record_locks {
            let whole_record_lock_guard = record_lock.get_whole_record_lock().write_arc().await;
            record_lock.set_record_purpose(purpose.clone());

            record_lock_guards.push(RecordLockGuard::new(record_lock, whole_record_lock_guard));
        }

        RecordsLockGuard::new(record_lock_guards)
    }

    pub fn try_lock_record(
        &self,
        record: OpaqueRecordKey,
        purpose: R,
    ) -> Option<RecordLockGuard<R, S>> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_lock = inner
                .record_lock_table
                .entry(record.clone())
                .or_insert_with(|| Arc::new(RecordLock::new(record.clone())));
            record_lock
        };

        // Wait on each lock to complete in order
        let whole_record_lock_guard = record_lock.get_whole_record_lock().try_write_arc()?;
        record_lock.set_record_purpose(purpose);

        Some(RecordLockGuard::new(record_lock, whole_record_lock_guard))
    }

    pub fn try_lock_records(
        &self,
        mut records: Vec<OpaqueRecordKey>,
        purpose: R,
    ) -> Option<RecordsLockGuard<R, S>> {
        // Always lock in sorted order to avoid deadlocks
        records.sort();

        // Get record locks
        let record_locks = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_locks = records
                .into_iter()
                .map(|record| {
                    inner
                        .record_lock_table
                        .entry(record.clone())
                        .or_insert_with(|| Arc::new(RecordLock::new(record.clone())))
                })
                .collect::<Vec<_>>();
            record_locks
        };

        // Wait on each lock to complete in order
        let mut record_lock_guards = vec![];
        for record_lock in record_locks {
            let whole_record_lock_guard = record_lock.get_whole_record_lock().try_write_arc()?;
            record_lock.set_record_purpose(purpose.clone());
            record_lock_guards.push(RecordLockGuard::new(record_lock, whole_record_lock_guard));
        }

        Some(RecordsLockGuard::new(record_lock_guards))
    }

    pub async fn lock_subkey(
        &self,
        record: OpaqueRecordKey,
        subkey: ValueSubkey,
        purpose: S,
    ) -> SubkeyLockGuard<R, S> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_lock = inner
                .record_lock_table
                .entry(record.clone())
                .or_insert_with(|| Arc::new(RecordLock::new(record.clone())));
            record_lock
        };

        // Attempt shared lock
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let whole_record_lock_guard = match timeout(30000, record_lock.get_whole_record_lock().read_arc()).await {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("active guards: {:#?}", record_lock.get_active_guards().lock().values().collect::<Vec<_>>());
                        panic!("lock_subkey deadlock");
                    }
                };
            } else {
                let whole_record_lock_guard = record_lock.get_whole_record_lock().read_arc().await;
            }
        }

        // Get subkey lock
        let subkey_lock = record_lock.get_subkey_lock(subkey);
        let subkey_lock_guard = asyncmutex_lock_arc!(subkey_lock);
        record_lock.set_subkey_purpose(subkey, purpose);

        SubkeyLockGuard::new(
            record_lock,
            whole_record_lock_guard,
            subkey_lock_guard,
            subkey,
        )
    }

    pub fn try_lock_subkey(
        &self,
        record: OpaqueRecordKey,
        subkey: ValueSubkey,
        purpose: S,
    ) -> Option<SubkeyLockGuard<R, S>> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_lock = inner
                .record_lock_table
                .entry(record.clone())
                .or_insert_with(|| Arc::new(RecordLock::new(record.clone())));
            record_lock
        };

        // Attempt shared lock
        let whole_record_lock_guard = record_lock.get_whole_record_lock().try_read_arc()?;

        // Get subkey lock
        let subkey_lock = record_lock.get_subkey_lock(subkey);
        let subkey_lock_guard = asyncmutex_try_lock_arc!(subkey_lock)?;
        record_lock.set_subkey_purpose(subkey, purpose);

        Some(SubkeyLockGuard::new(
            record_lock,
            whole_record_lock_guard,
            subkey_lock_guard,
            subkey,
        ))
    }

    pub async fn peek_lock(&self, record: OpaqueRecordKey) -> PeekLockGuard<R, S> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_lock = inner
                .record_lock_table
                .entry(record.clone())
                .or_insert_with(|| Arc::new(RecordLock::new(record.clone())));
            record_lock
        };

        // Attempt shared lock
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let whole_record_lock_guard = match timeout(30000, record_lock.get_whole_record_lock().read_arc()).await {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("active guards: {:#?}", record_lock.get_active_guards().lock().values().collect::<Vec<_>>());
                        panic!("peek_lock deadlock");
                    }
                };
            } else {
                let whole_record_lock_guard = record_lock.get_whole_record_lock().read_arc().await;
            }
        }

        record_lock.add_peek();

        PeekLockGuard::new(record_lock, whole_record_lock_guard)
    }

    pub fn try_peek_lock(&self, record: OpaqueRecordKey) -> Option<PeekLockGuard<R, S>> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let record_lock = inner
                .record_lock_table
                .entry(record.clone())
                .or_insert_with(|| Arc::new(RecordLock::new(record.clone())));
            record_lock
        };

        // Attempt shared lock
        let whole_record_lock_guard = record_lock.get_whole_record_lock().try_read_arc()?;
        record_lock.add_peek();

        Some(PeekLockGuard::new(record_lock, whole_record_lock_guard))
    }

    pub fn get_record_lock_kind(&self, record: &OpaqueRecordKey) -> RecordLockKind<R, S> {
        // Get record lock
        let record_lock = {
            let mut inner = self.inner.lock();
            inner.record_lock_table.remove_expired();
            let Some(record_lock) = inner.record_lock_table.get(record) else {
                return RecordLockKind::Unlocked;
            };
            record_lock
        };

        // Attempt shared lock
        let Some(_whole_record_lock_guard) = record_lock.get_whole_record_lock().try_read_arc()
        else {
            // If we can't read lock it, then it is whole-record locked, so return the whole range as locked
            return RecordLockKind::RecordLocked {
                purpose: record_lock.purpose().unwrap(),
            };
        };

        RecordLockKind::SubkeyLocked {
            purpose_map: record_lock.get_subkey_purpose_map(),
            peek_count: record_lock.get_peek_count(),
        }
    }
}
