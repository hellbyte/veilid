mod commit_action;
mod debug;
mod inbound_transactions;
mod inbound_transactions_interface;
mod inbound_watches;
mod inbound_watches_interface;
mod keys;
mod limited_size;
mod load_action;
mod record_data;
mod record_index;

use super::*;

pub(super) use commit_action::*;
pub(in super::super) use inbound_transactions::*;
pub(super) use inbound_transactions_interface::*;
pub(in super::super) use inbound_watches::*;
pub(super) use keys::*;
pub(super) use limited_size::*;
pub(super) use load_action::*;
pub(super) use record_data::*;
pub(super) use record_index::*;

/// Mutable record store state
pub(super) struct RecordStoreInner<D>
where
    D: RecordDetail,
{
    unlocked_inner: Arc<RecordStoreUnlockedInner>,

    /// In-memory record index and cache
    record_index: RecordIndex<D>,

    /// The watches per record
    inbound_watches: InboundWatches,

    /// The transactions per record
    inbound_transactions: InboundTransactions,
}

impl<D> VeilidComponentRegistryAccessor for RecordStoreInner<D>
where
    D: RecordDetail,
{
    fn registry(&self) -> VeilidComponentRegistry {
        self.unlocked_inner.registry.clone()
    }
}

impl<D> fmt::Debug for RecordStoreInner<D>
where
    D: RecordDetail,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecordStoreInner")
            .field("record_index", &self.record_index)
            .field("inbound_watches", &self.inbound_watches)
            .field("inbound_transactions", &self.inbound_transactions)
            .finish()
    }
}

impl<D> RecordStoreInner<D>
where
    D: RecordDetail,
{
    pub async fn try_new(unlocked_inner: Arc<RecordStoreUnlockedInner>) -> EyreResult<Self> {
        let record_index = RecordIndex::try_new(unlocked_inner.clone()).await?;

        Ok(Self {
            record_index,
            inbound_watches: InboundWatches::new(),
            inbound_transactions: InboundTransactions::new(),
            unlocked_inner,
        })
    }

    pub fn new_record(
        &mut self,
        opaque_record_key: OpaqueRecordKey,
        record: Record<D>,
    ) -> VeilidAPIResult<Option<CommitAction<D>>> {
        self.record_index
            .create_record(opaque_record_key.clone(), record)?;
        Ok(self.record_index.maybe_prepare_commit_action())
    }

    pub fn delete_record(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
    ) -> VeilidAPIResult<Option<CommitAction<D>>> {
        self.record_index.delete_record(opaque_record_key.clone())?;
        self.cleanup_record(opaque_record_key);
        Ok(self.record_index.maybe_prepare_commit_action())
    }

    pub fn set_single_subkey(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        subkey: ValueSubkey,
        value: Arc<SignedValueData>,
        watch_update_mode: &InboundWatchUpdateMode,
    ) -> VeilidAPIResult<Option<CommitAction<D>>> {
        self.record_index
            .set_single_subkey(opaque_record_key, subkey, value)?;

        // Update watches
        self.update_watched_value(opaque_record_key, subkey, watch_update_mode);

        // Single subkeys on a single record can be committed lazily
        Ok(self.record_index.maybe_prepare_commit_action())
    }

    pub fn set_subkeys_single_record(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        subkey_values: &SubkeyValueList,
        watch_update_mode: &InboundWatchUpdateMode,
    ) -> VeilidAPIResult<Option<CommitAction<D>>> {
        self.record_index
            .set_subkeys_single_record(opaque_record_key, subkey_values)?;

        // Update watches
        for subkey in subkey_values.iter().map(|x| x.0) {
            self.update_watched_value(opaque_record_key, subkey, watch_update_mode);
        }

        // Multiple subkeys on a single record should be committed immediately to ensure they all commit together
        Ok(self.record_index.prepare_commit_action())
    }

    pub fn set_subkeys_multiple_records(
        &mut self,
        keys_and_subkeys: &RecordSubkeyValueList,
        watch_update_mode: &InboundWatchUpdateMode,
    ) -> VeilidAPIResult<Option<CommitAction<D>>> {
        self.record_index
            .set_subkeys_multiple_records(keys_and_subkeys)?;

        // Update watches
        for (opaque_record_key, subkey_values) in keys_and_subkeys.iter() {
            for subkey in subkey_values.iter().map(|x| x.0) {
                self.update_watched_value(opaque_record_key, subkey, watch_update_mode);
            }
        }

        // Multiple subkeys on multiple records should be committed immediately to ensure they all commit together
        Ok(self.record_index.prepare_commit_action())
    }

    pub fn flush(&mut self) -> Option<CommitAction<D>> {
        self.record_index.prepare_commit_action()
    }

    pub fn finish_commit_action(&mut self, commit_action: CommitAction<D>) -> VeilidAPIResult<()> {
        self.record_index.finish_commit_action(commit_action)
    }

    pub fn prepare_get_load_action(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        subkey: ValueSubkey,
    ) -> LoadActionResult {
        self.record_index
            .prepare_load_action(opaque_record_key.clone(), subkey, false)
    }

    pub fn prepare_peek_load_action(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        subkey: ValueSubkey,
    ) -> LoadActionResult {
        self.record_index
            .prepare_load_action(opaque_record_key.clone(), subkey, true)
    }

    pub fn finish_load_action(&mut self, load_action: LoadAction) {
        self.record_index.finish_load_action(load_action);
    }

    pub fn contains_record(&self, opaque_record_key: &OpaqueRecordKey) -> bool {
        self.record_index.contains_record(opaque_record_key)
    }

    pub fn peek_record<F, R>(&self, opaque_record_key: &OpaqueRecordKey, func: F) -> Option<R>
    where
        F: FnOnce(&Record<D>) -> R,
    {
        self.record_index.peek_record(opaque_record_key, func)
    }

    pub fn with_record<F, R>(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        func: F,
    ) -> VeilidAPIResult<Option<R>>
    where
        F: FnOnce(&Record<D>) -> R,
    {
        self.record_index.with_record(opaque_record_key, func)
    }

    pub fn with_record_detail_mut<R, F>(
        &mut self,
        opaque_record_key: &OpaqueRecordKey,
        func: F,
    ) -> VeilidAPIResult<Option<R>>
    where
        F: FnOnce(Arc<SignedValueDescriptor>, &mut D) -> R,
    {
        self.record_index
            .with_record_detail_mut(opaque_record_key, func)
    }

    pub fn reclaim_space(&mut self, space: u64) -> (Option<CommitAction<D>>, ReclaimedSpace) {
        let mut total_reclaimed_space = ReclaimedSpace {
            reclaimed: 0,
            total: self.record_index.total_storage_space(),
            dead_records: vec![],
        };

        while total_reclaimed_space.reclaimed < space {
            let mut reclaimed_space = match self.record_index.delete_lru() {
                Ok(v) => v,
                Err(e) => {
                    veilid_log!(self error "Error reclaiming space: {}", e);
                    break;
                }
            };

            if reclaimed_space.dead_records.is_empty() {
                break;
            }

            for dead_record in &reclaimed_space.dead_records {
                self.cleanup_record(dead_record);
            }

            total_reclaimed_space.reclaimed += reclaimed_space.reclaimed;
            total_reclaimed_space.total = reclaimed_space.total;
            total_reclaimed_space
                .dead_records
                .append(&mut reclaimed_space.dead_records);
        }

        (
            self.record_index.prepare_commit_action(),
            total_reclaimed_space,
        )
    }

    ////////////////////////////////////////////////////////////

    fn cleanup_record(&mut self, opaque_record_key: &OpaqueRecordKey) {
        if self
            .record_index
            .peek_record(opaque_record_key, |_| {})
            .is_some()
        {
            veilid_log!(self error "Record should not exist in index: {}", opaque_record_key);
            return;
        }

        let rtk = RecordTableKey {
            record_key: opaque_record_key.clone(),
        };

        // Remove transactions
        let _ = self
            .inbound_transactions
            .try_remove_record(&rtk)
            .inspect_err(veilid_log_err!(self));

        // Remove watches
        let _ = self
            .inbound_watches
            .try_remove_record(&rtk)
            .inspect_err(veilid_log_err!(self));
    }
}
