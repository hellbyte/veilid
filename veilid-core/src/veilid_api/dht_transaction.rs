use super::*;
use crate::storage_manager::OutboundTransactionHandle;

impl_veilid_log_facility!("veilid_api");

///////////////////////////////////////////////////////////////////////////////////////

/// DHT Transactions the way you perform multiple simulateous atomic operations over a set of DHT records.
///
/// DHT operations performed out of a transaction may be processed in any order, and only operate on one subkey at a time
/// for a given record. Transactions allow you to bind a set of operations so they all succeed, or fail together, and at the same time.
///
/// Transactional DHT operations can only be performed when the node is online, and will error with [VeilidAPIError::TryAgain] if offline.
///
/// Transactions must be committed when all of their operations are registered, or rolled back if the group of operations is to be cancelled.
#[derive(Clone)]
#[must_use]
pub struct DHTTransaction {
    /// API in use
    api: VeilidAPI,
    /// Inner transaction
    inner: Arc<Mutex<DHTTransactionInner>>,
}

impl fmt::Debug for DHTTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DHTTransaction")
            .field("handle", &self.inner.lock().opt_transaction_handle)
            .finish()
    }
}

impl DHTTransaction {
    ////////////////////////////////////////////////////////////////

    pub(super) fn new(api: VeilidAPI, handle: OutboundTransactionHandle) -> VeilidAPIResult<Self> {
        let registry = api.core_context()?.registry();
        Ok(Self {
            api,
            inner: Arc::new(Mutex::new(DHTTransactionInner {
                registry,
                opt_transaction_handle: Some(handle),
            })),
        })
    }

    /// Get the [VeilidAPI] object that created this [DHTTransaction].
    pub fn api(&self) -> VeilidAPI {
        self.api.clone()
    }

    #[must_use]
    pub(crate) fn log_key(&self) -> &str {
        self.api.log_key()
    }

    /// Commit the transaction
    /// All write operations are performed atomically
    #[cfg_attr(feature = "instrument", instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key(), transaction_handle), skip(self), ret))]
    pub async fn commit(self) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            let storage_manager = self.api.core_context()?.storage_manager();
            let transaction_handle = {
                let mut inner = self.inner.lock();
                inner
                    .opt_transaction_handle
                    .take()
                    .ok_or_else(|| VeilidAPIError::generic("transaction already completed"))?
            };

            tracing::Span::current().record("transaction_handle", transaction_handle.to_string());

            veilid_log!(self debug
                "DHTTransaction::commit(transaction_handle: {}", transaction_handle);

            // End and commit transaction
            Box::pin(storage_manager.end_and_commit_transaction(transaction_handle)).await
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Rollback the transaction
    /// No write operations are performed,
    #[cfg_attr(feature = "instrument", instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key(), transaction_handle), skip(self), ret))]
    pub async fn rollback(self) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            let storage_manager = self.api.core_context()?.storage_manager();
            let transaction_handle = {
                let mut inner = self.inner.lock();
                inner
                    .opt_transaction_handle
                    .take()
                    .ok_or_else(|| VeilidAPIError::generic("transaction already completed"))?
            };

            tracing::Span::current().record("transaction_handle", transaction_handle.to_string());

            veilid_log!(self debug
                "DHTTransaction::rollback(transaction_handle: {}", transaction_handle);

            Box::pin(storage_manager.rollback_transaction(transaction_handle)).await
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Add a set_dht_value operation to the transaction
    ///
    /// * Will fail if performed offline
    /// * Will fail if existing offline writes exist for this record key
    ///
    /// The writer, if specified, will override the 'default_writer' specified when the record is opened.
    ///
    /// Returns `None` if the value was successfully set.
    /// Returns `Some(data)` if the value set was older than the one available on the network.
    #[cfg_attr(feature = "instrument", instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key(), transaction_handle, data = print_data(&data, Some(64))), skip(self, data), ret))]
    pub async fn set(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
        data: Vec<u8>,
        options: Option<DHTTransactionSetValueOptions>,
    ) -> VeilidAPIResult<Option<ValueData>> {
        record_duration_fut(async {
            let storage_manager = self.api.core_context()?.storage_manager();
            let transaction_handle = {
                let inner = self.inner.lock();
                inner
                    .opt_transaction_handle
                    .clone()
                    .ok_or_else(|| VeilidAPIError::generic("transaction already completed"))?
            };

            tracing::Span::current().record("transaction_handle", transaction_handle.to_string());

            veilid_log!(self debug
                "DHTTransaction::set(transaction_handle: {}, key: {}, subkey: {}, data: len={}, options: {:?})", transaction_handle, record_key, subkey, data.len(), options);

            storage_manager.check_record_key(&record_key)?;

            Box::pin(storage_manager.transaction_set(
                transaction_handle,
                record_key,
                subkey,
                data,
                options,
            ))
            .await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Perform a get_dht_value operation inside the transaction
    ///
    /// * Will fail if performed offline
    /// * Will pull the latest value from the network, will fail if the local value is newer
    /// * Will fail if existing offline writes exist for this record key
    ///
    /// Returns `None` if the value subkey has not yet been set.
    /// Returns `Some(data)` if the value subkey has valid data.
    #[cfg_attr(feature = "instrument", instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret))]
    pub async fn get(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
    ) -> VeilidAPIResult<Option<ValueData>> {
        record_duration_fut(async {
            let storage_manager = self.api.core_context()?.storage_manager();
            let transaction_handle = {
                let inner = self.inner.lock();
                inner
                    .opt_transaction_handle
                    .clone()
                    .ok_or_else(|| VeilidAPIError::generic("transaction already completed"))?
            };

            tracing::Span::current().record("transaction_handle", transaction_handle.to_string());

            veilid_log!(self debug
                "DHTTransaction::get(transaction_handle: {}, key: {}, subkey: {})", transaction_handle, record_key, subkey);

            storage_manager.check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.transaction_get(transaction_handle, record_key, subkey)).await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Perform a inspect_dht_record operation inside the transaction
    ///
    /// * Does not perform any network activity, as the transaction state keeps all of the required information after the begin
    ///
    /// For information on arguments, see [RoutingContext::inspect_dht_record]
    ///
    /// Returns a DHTRecordReport with the subkey ranges that were returned that overlapped the schema, and sequence numbers for each of the subkeys in the range.
    #[cfg_attr(feature = "instrument", instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key(), transaction_handle), skip(self), ret))]
    pub async fn inspect(
        &self,
        record_key: RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        scope: DHTReportScope,
    ) -> VeilidAPIResult<DHTRecordReport> {
        record_duration_fut(async {
            let storage_manager = self.api.core_context()?.storage_manager();
            let transaction_handle = {
                let inner = self.inner.lock();
                inner
                    .opt_transaction_handle
                    .clone()
                    .ok_or_else(|| VeilidAPIError::generic("transaction already completed"))?
            };

            tracing::Span::current().record("transaction_handle", transaction_handle.to_string());

            veilid_log!(self debug
                "DHTTransaction::inspect(transaction_handle: {}, record_key: {}, subkeys: {}, scope: {:?})", transaction_handle, record_key, subkeys.as_ref().map(|x| x.to_string()).unwrap_or_else(|| "None".to_string()), scope);

            storage_manager.check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            storage_manager.transaction_inspect(
                transaction_handle,
                record_key,
                subkeys,
                scope,
            )
        }).await.inspect_err(log_veilid_api_error!(self))
    }
}
//////////////////////////////////////////////////////////////////////////////////////

struct DHTTransactionInner {
    registry: VeilidComponentRegistry,
    opt_transaction_handle: Option<OutboundTransactionHandle>,
}

impl Drop for DHTTransactionInner {
    fn drop(&mut self) {
        if let Some(transaction_handle) = self.opt_transaction_handle.take() {
            let registry = &self.registry;
            veilid_log!(registry warn "Dropped DHT transaction without commit or rollback");

            let storage_manager = registry.storage_manager();
            storage_manager.drop_transaction_sync(transaction_handle);
        }
    }
}
