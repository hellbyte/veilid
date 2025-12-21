use super::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StorageManagerRecordLockPurpose {
    Create,
    Open,
    Close,
    Delete,
    Watch,
    TransactBegin,
    TransactEndAndCommit,
    TransactRollback,
    TransactDrop,
}

impl LockPurpose for StorageManagerRecordLockPurpose {}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StorageManagerSubkeyLockPurpose {
    Get,
    Set,
    TransactGet,
    TransactSet,
}

impl LockPurpose for StorageManagerSubkeyLockPurpose {}

pub type StorageManagerRecordLockTable =
    RecordLockTable<StorageManagerRecordLockPurpose, StorageManagerSubkeyLockPurpose>;

pub type StorageManagerRecordLockGuard =
    RecordLockGuard<StorageManagerRecordLockPurpose, StorageManagerSubkeyLockPurpose>;
pub type StorageManagerRecordsLockGuard =
    RecordsLockGuard<StorageManagerRecordLockPurpose, StorageManagerSubkeyLockPurpose>;
pub type StorageManagerSubkeyLockGuard =
    SubkeyLockGuard<StorageManagerRecordLockPurpose, StorageManagerSubkeyLockPurpose>;
pub type StorageManagerPeekLockGuard =
    PeekLockGuard<StorageManagerRecordLockPurpose, StorageManagerSubkeyLockPurpose>;
