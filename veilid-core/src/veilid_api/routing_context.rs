use super::*;

impl_veilid_log_facility!("veilid_api");

///////////////////////////////////////////////////////////////////////////////////////

/// Valid destinations for a message sent over a routing context.
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi, namespace)
)]
#[must_use]
pub enum Target {
    /// Node by its node id
    #[schemars(with = "String")]
    NodeId(NodeId),
    /// Remote private route by its id.
    #[schemars(with = "String")]
    RouteId(RouteId),
}

pub(crate) struct RoutingContextUnlockedInner {
    /// Safety routing requirements.
    safety_selection: SafetySelection,
}

/// Routing contexts are the way you specify the communication preferences for Veilid.
///
/// By default routing contexts have 'safety routing' enabled which offers sender privacy.
/// privacy. To disable this and send RPC operations straight from the node use [RoutingContext::with_safety()] with a [SafetySelection::Unsafe] parameter.
/// To enable receiver privacy, you should send to a private route RouteId that you have imported, rather than directly to a NodeId.
///
#[derive(Clone)]
#[must_use]
pub struct RoutingContext {
    /// Veilid API handle.
    api: VeilidAPI,
    unlocked_inner: Arc<RoutingContextUnlockedInner>,
}

impl fmt::Debug for RoutingContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoutingContext")
            .field("ptr", &format!("{:p}", Arc::as_ptr(&self.unlocked_inner)))
            .field("safety_selection", &self.unlocked_inner.safety_selection)
            .finish()
    }
}

impl RoutingContext {
    ////////////////////////////////////////////////////////////////

    pub(super) fn try_new(api: VeilidAPI) -> VeilidAPIResult<Self> {
        let config = api.config()?;

        Ok(Self {
            api,
            unlocked_inner: Arc::new(RoutingContextUnlockedInner {
                safety_selection: SafetySelection::Safe(SafetySpec {
                    preferred_route: None,
                    hop_count: config.network.rpc.default_route_hop_count as usize,
                    stability: Stability::Reliable,
                    sequencing: Sequencing::PreferOrdered,
                }),
            }),
        })
    }

    #[must_use]
    pub(crate) fn log_key(&self) -> &str {
        self.api.log_key()
    }

    /// Turn on sender privacy, enabling the use of safety routes. This is the default and
    /// calling this function is only necessary if you have previously disable safety or used other parameters.
    ///
    /// Default values for hop count, stability and sequencing preferences are used.
    ///
    /// * Hop count default is dependent on config, but is set to 1 extra hop.
    /// * Stability default is to choose reliable routes, preferring them over low latency.
    /// * Sequencing default is to prefer ordered before unordered message delivery.
    ///
    /// To customize the safety selection in use, use [RoutingContext::with_safety()].
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub fn with_default_safety(self) -> VeilidAPIResult<Self> {
        let this = self.clone();
        record_duration(|| {
            veilid_log!(self debug
            "RoutingContext::with_default_safety(self: {:?})", self);

            let config = self.api.config()?;

            self.with_safety(SafetySelection::Safe(SafetySpec {
                preferred_route: None,
                hop_count: config.network.rpc.default_route_hop_count as usize,
                stability: Stability::Reliable,
                sequencing: Sequencing::PreferOrdered,
            }))
        })
        .inspect_err(log_veilid_api_error!(this))
    }

    /// Use a custom [SafetySelection]. Can be used to disable safety via [SafetySelection::Unsafe].
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub fn with_safety(self, safety_selection: SafetySelection) -> VeilidAPIResult<Self> {
        let this = self.clone();
        record_duration(|| {
            veilid_log!(self debug
            "RoutingContext::with_safety(self: {:?}, safety_selection: {:?})", self, safety_selection);

            if let SafetySelection::Unsafe(_) = &safety_selection {
                #[cfg(not(feature = "footgun"))]
                {
                    return Err(VeilidAPIError::generic(
                        "Unsafe routing mode is not allowed without the 'footgun' feature enabled",
                    ));
                }
            }

            if let SafetySelection::Safe(safe) = &safety_selection {
                if let Some(preferred_route) = &safe.preferred_route {
                    self.api
                        .core_context()?
                        .routing_table()
                        .check_route_id(preferred_route)?;
                }
            }

            Ok(Self {
                api: self.api.clone(),
                unlocked_inner: Arc::new(RoutingContextUnlockedInner { safety_selection }),
            })
        }).inspect_err(log_veilid_api_error!(this))
    }

    /// Use a specified [Sequencing] preference, with or without privacy.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub fn with_sequencing(self, sequencing: Sequencing) -> Self {
        record_duration(|| {
            veilid_log!(self debug
            "RoutingContext::with_sequencing(self: {:?}, sequencing: {:?})", self, sequencing);

            Self {
                api: self.api.clone(),
                unlocked_inner: Arc::new(RoutingContextUnlockedInner {
                    safety_selection: match &self.unlocked_inner.safety_selection {
                        SafetySelection::Unsafe(_) => SafetySelection::Unsafe(sequencing),
                        SafetySelection::Safe(safety_spec) => SafetySelection::Safe(SafetySpec {
                            preferred_route: safety_spec.preferred_route.clone(),
                            hop_count: safety_spec.hop_count,
                            stability: safety_spec.stability,
                            sequencing,
                        }),
                    },
                }),
            }
        })
    }

    /// Get the safety selection in use on this routing context.
    pub fn safety(&self) -> SafetySelection {
        self.unlocked_inner.safety_selection.clone()
    }

    /// Get the sequencing used by this routing context
    pub fn sequencing(&self) -> Sequencing {
        match &self.unlocked_inner.safety_selection {
            SafetySelection::Unsafe(sequencing) => *sequencing,
            SafetySelection::Safe(safety_spec) => safety_spec.sequencing,
        }
    }

    /// Get the [VeilidAPI] object that created this [RoutingContext].
    pub fn api(&self) -> VeilidAPI {
        self.api.clone()
    }

    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    async fn get_destination(&self, target: Target) -> VeilidAPIResult<rpc_processor::Destination> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::get_destination(self: {:?}, target: {:?})", self, target);

            let rpc_processor = self.api.core_context()?.rpc_processor();
            Box::pin(rpc_processor.resolve_target_to_destination(
                target,
                self.unlocked_inner.safety_selection.clone(),
            ))
            .await
            .map_err(VeilidAPIError::invalid_target)
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    fn check_target(&self, target: &Target) -> VeilidAPIResult<()> {
        match target {
            Target::NodeId(node_id) => {
                self.api
                    .core_context()?
                    .routing_table()
                    .check_node_id(node_id)?;
            }
            Target::RouteId(route_id) => {
                self.api
                    .core_context()?
                    .routing_table()
                    .check_route_id(route_id)?;
            }
        }
        Ok(())
    }

    ////////////////////////////////////////////////////////////////
    /// App-level Messaging

    #[instrument(target = "veilid_api", level = "debug", skip(message), fields(duration, __VEILID_LOG_KEY = self.log_key(), message_len = message.len(), ret.len))]
    async fn internal_app_call(
        &self,
        target: Target,
        message: Vec<u8>,
    ) -> VeilidAPIResult<Vec<u8>> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::app_call(self: {:?}, target: {:?}, message_len: {:?})", self, target, message.len());
            veilid_log!(self trace "message: {:?}", message);

            self.check_target(&target)?;

            let rpc_processor = self.api.core_context()?.rpc_processor();

            // Get destination
            let dest = self.get_destination(target).await?;

            // Send app message
            let answer = match Box::pin(rpc_processor.rpc_call_app_call(dest, message)).await {
                Ok(NetworkResult::Value(v)) => v,
                Ok(NetworkResult::Timeout) => apibail_timeout!(),
                Ok(NetworkResult::ServiceUnavailable(e)) => apibail_invalid_target!(e),
                Ok(NetworkResult::NoConnection(e)) | Ok(NetworkResult::AlreadyExists(e)) => {
                    apibail_no_connection!(e);
                }

                Ok(NetworkResult::InvalidMessage(message)) => {
                    apibail_generic!(message);
                }
                Err(e) => return Err(e.into()),
            };

            tracing::Span::current().record("ret.len", answer.answer.len());

            Ok(answer.answer)
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    #[cfg(feature = "footgun")]
    /// App-level bidirectional call that expects a response to be returned.
    ///
    /// Veilid apps may use this for arbitrary message passing.
    ///
    /// * `target` - can be either a direct node id or a private route.
    /// * `message` - an arbitrary message blob of up to 32768 bytes.
    ///
    /// Returns an answer blob of up to 32768 bytes.
    pub async fn app_call(&self, target: Target, message: Vec<u8>) -> VeilidAPIResult<Vec<u8>> {
        self.internal_app_call(target, message).await
    }

    #[cfg(not(feature = "footgun"))]
    /// App-level bidirectional call that expects a response to be returned.
    ///
    /// Veilid apps may use this for arbitrary message passing.
    ///
    /// * `target` - a private route id
    /// * `message` - an arbitrary message blob of up to 32768 bytes.
    ///
    /// Returns an answer blob of up to 32768 bytes.
    pub async fn app_call(&self, target: Target, message: Vec<u8>) -> VeilidAPIResult<Vec<u8>> {
        match target {
            Target::RouteId(_) => self.internal_app_call(target, message).await,
            Target::NodeId(_) => Err(VeilidAPIError::invalid_target(
                "Only RouteId targets are allowed without the footgun feature",
            )),
        }
    }

    #[instrument(target = "veilid_api", level = "debug", skip(message), fields(duration, __VEILID_LOG_KEY = self.log_key(), message_len = message.len()), ret)]
    async fn internal_app_message(&self, target: Target, message: Vec<u8>) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::app_message(self: {:?}, target: {:?}, message_len: {})", self, target, message.len());
            veilid_log!(self trace "message: {:?}", message);

            self.check_target(&target)?;

            let rpc_processor = self.api.core_context()?.rpc_processor();

            // Get destination
            let dest = self.get_destination(target).await?;

            // Send app message
            match Box::pin(rpc_processor.rpc_call_app_message(dest, message)).await {
                Ok(NetworkResult::Value(())) => {}
                Ok(NetworkResult::Timeout) => apibail_timeout!(),
                Ok(NetworkResult::ServiceUnavailable(e)) => apibail_invalid_target!(e),
                Ok(NetworkResult::NoConnection(e)) | Ok(NetworkResult::AlreadyExists(e)) => {
                    apibail_no_connection!(e);
                }
                Ok(NetworkResult::InvalidMessage(message)) => {
                    apibail_generic!(message);
                }
                Err(e) => return Err(e.into()),
            };

            Ok(())
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    #[cfg(feature = "footgun")]
    /// App-level unidirectional message that does not expect any value to be returned.
    ///
    /// Veilid apps may use this for arbitrary message passing.
    ///
    /// * `target` - can be either a direct node id or a private route.
    /// * `message` - an arbitrary message blob of up to 32768 bytes.
    pub async fn app_message(&self, target: Target, message: Vec<u8>) -> VeilidAPIResult<()> {
        self.internal_app_message(target, message).await
    }

    #[cfg(not(feature = "footgun"))]
    /// App-level unidirectional message that does not expect any value to be returned.
    ///
    /// Veilid apps may use this for arbitrary message passing.
    ///
    /// * `target` - a private route.
    /// * `message` - an arbitrary message blob of up to 32768 bytes.
    pub async fn app_message(&self, target: Target, message: Vec<u8>) -> VeilidAPIResult<()> {
        match target {
            Target::RouteId(_) => self.internal_app_message(target, message).await,
            Target::NodeId(_) => Err(VeilidAPIError::invalid_target(
                "Only PrivateRoute targets are allowed without the footgun feature",
            )),
        }
    }

    ///////////////////////////////////
    // DHT Records

    /// Creates a new DHT record
    ///
    /// The record is considered 'open' after the create operation succeeds.
    /// * 'kind' - specify a cryptosystem kind to use
    /// * 'schema' - the schema to use when creating the DHT record
    /// * 'owner' - optionally specify an owner keypair to use. If you leave this as None then a random one will be generated. If specified, the crypto kind of the owner must match that of the `kind` parameter
    /// Returns the newly allocated DHT record's key if successful.
    ///
    /// Note: if you pass in an owner keypair this call is a deterministic! This means that if you try to create a new record for a given owner and schema that already exists it *will* fail.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn create_dht_record(
        &self,
        kind: CryptoKind,
        schema: DHTSchema,
        owner: Option<KeyPair>,
    ) -> VeilidAPIResult<DHTRecordDescriptor> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::create_dht_record(self: {:?}, schema: {:?}, owner: {:?}, kind: {:?})", self, schema, owner, kind);
            Crypto::validate_crypto_kind(kind)?;
            schema.validate()?;
            if let Some(owner) = &owner {
                self.api.crypto()?.check_keypair(owner)?;
            }

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.create_record(
                kind,
                schema,
                owner,
                self.unlocked_inner.safety_selection.clone(),
            ))
            .await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Opens a DHT record at a specific key.
    ///
    /// Associates a 'default_writer' secret if one is provided to provide writer capability. The
    /// writer can be overridden if specified here via the set_dht_value writer.
    ///
    /// Records may only be opened or created. If a record is re-opened it will use the new writer and routing context
    /// ignoring the settings of the last time it was opened. This allows one to open a record a second time
    /// without first closing it, which will keep the active 'watches' on the record but change the default writer or
    /// safety selection.
    ///
    /// Returns the DHT record descriptor for the opened record if successful.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn open_dht_record(
        &self,
        record_key: RecordKey,
        default_writer: Option<KeyPair>,
    ) -> VeilidAPIResult<DHTRecordDescriptor> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::open_dht_record(self: {:?}, key: {:?}, default_writer: {:?})", self, record_key, default_writer);

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;
            if let Some(default_writer) = &default_writer {
                self.api.crypto()?.check_keypair(default_writer)?;
            }

            let storage_manager = self.api.core_context()?.storage_manager();
            storage_manager
                .open_record(
                    record_key,
                    default_writer,
                    self.unlocked_inner.safety_selection.clone(),
                )
                .await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Closes a DHT record at a specific key that was opened with create_dht_record or open_dht_record.
    ///
    /// Closing a record allows you to re-open it with a different routing context.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn close_dht_record(&self, record_key: RecordKey) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::close_dht_record(self: {:?}, key: {:?})", self, record_key);

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.close_record(record_key)).await
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Deletes a DHT record at a specific key.
    ///
    /// If the record is opened, it must be closed before it is deleted.
    /// Deleting a record does not delete it from the network, but will remove the storage of the record
    /// locally, and will prevent its value from being refreshed on the network by this node.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn delete_dht_record(&self, record_key: RecordKey) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::delete_dht_record(self: {:?}, key: {:?})", self, record_key);

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.delete_record(record_key)).await
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Gets the latest value of a subkey.
    ///
    /// May pull the latest value from the network, but by setting 'force_refresh' you can force a network data refresh.
    ///
    /// Returns `None` if the value subkey has not yet been set.
    /// Returns `Some(data)` if the value subkey has valid data.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn get_dht_value(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
        force_refresh: bool,
    ) -> VeilidAPIResult<Option<ValueData>> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::get_dht_value(self: {:?}, key: {:?}, subkey: {:?}, force_refresh: {:?})", self, record_key, subkey, force_refresh);

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.get_value(record_key, subkey, force_refresh)).await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Pushes a changed subkey value to the network.
    /// The DHT record must first by opened via open_dht_record or create_dht_record.
    ///
    /// The writer, if specified, will override the 'default_writer' specified when the record is opened.
    ///
    /// Returns `None` if the value was successfully set.
    /// Returns `Some(data)` if the value set was older than the one available on the network.
    #[instrument(target = "veilid_api", level = "debug", skip(data), fields(duration, __VEILID_LOG_KEY = self.log_key(), data = print_data(&data, Some(64))), ret)]
    pub async fn set_dht_value(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
        data: Vec<u8>,
        options: Option<SetDHTValueOptions>,
    ) -> VeilidAPIResult<Option<ValueData>> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::set_dht_value(self: {:?}, key: {:?}, subkey: {:?}, data: len={}, options: {:?})", self, record_key, subkey, data.len(), options);

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.set_value(record_key, subkey, data, options)).await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Add or update a watch to a DHT value that informs the user via an VeilidUpdate::ValueChange callback when the record has subkeys change.
    /// One remote node will be selected to perform the watch and it will offer an expiration time based on a suggestion, and make an attempt to
    /// continue to report changes via the callback. Nodes that agree to doing watches will be put on our 'ping' list to ensure they are still around
    /// otherwise the watch will be cancelled and will have to be re-watched.
    ///
    /// There is only one watch permitted per record. If a change to a watch is desired, the previous one will be overwritten.
    /// * `key` is the record key to watch. it must first be opened for reading or writing.
    /// * `subkeys`:
    ///   - None: specifies watching the entire range of subkeys.
    ///   - Some(range): is the the range of subkeys to watch. The range must not exceed 512 discrete non-overlapping or adjacent subranges. If no range is specified, this is equivalent to watching the entire range of subkeys.
    /// * `expiration`:
    ///   - None: specifies a watch with no expiration
    ///   - Some(timestamp): the desired timestamp of when to automatically terminate the watch, in microseconds. If this value is less than `network.rpc.timeout_ms` milliseconds in the future, this function will return an error immediately.
    /// * `count:
    ///   - None: specifies a watch count of u32::MAX
    ///   - Some(count): is the number of times the watch will be sent, maximum. A zero value here is equivalent to a cancellation.
    ///
    /// Returns Ok(true) if a watch is active for this record.
    /// Returns Ok(false) if the entire watch has been cancelled.
    ///
    /// DHT watches are accepted with the following conditions:
    /// * First-come first-served basis for arbitrary unauthenticated readers, up to network.dht.public_watch_limit per record.
    /// * If a member (either the owner or a SMPL schema member) has opened the key for writing (even if no writing is performed) then the watch will be signed and guaranteed network.dht.member_watch_limit per writer.
    ///
    /// Members can be specified via the SMPL schema and do not need to allocate writable subkeys in order to offer a member watch capability.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn watch_dht_values(
        &self,
        record_key: RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        expiration: Option<Timestamp>,
        count: Option<u32>,
    ) -> VeilidAPIResult<bool> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::watch_dht_values(self: {:?}, key: {:?}, subkeys: {:?}, expiration: {:?}, count: {:?})", self, record_key, subkeys, expiration, count);
            let subkeys = subkeys.unwrap_or_default();
            let expiration = expiration.unwrap_or_default();
            let count = count.unwrap_or(u32::MAX);

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.watch_values(record_key, subkeys, expiration, count)).await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Cancels a watch early.
    ///
    /// This is a convenience function that cancels watching all subkeys in a range. The subkeys specified here
    /// are subtracted from the currently-watched subkey range.
    /// * `subkeys`:
    ///   - None: specifies watching the entire range of subkeys.
    ///   - Some(range): is the the range of subkeys to watch. The range must not exceed 512 discrete non-overlapping or adjacent subranges. If no range is specified, this is equivalent to watching the entire range of subkeys.
    /// Only the subkey range is changed, the expiration and count remain the same.
    /// If no subkeys remain, the watch is entirely cancelled and will receive no more updates.
    ///
    /// Returns Ok(true) if a watch is active for this record.
    /// Returns Ok(false) if the entire watch has been cancelled.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn cancel_dht_watch(
        &self,
        record_key: RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
    ) -> VeilidAPIResult<bool> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::cancel_dht_watch(self: {:?}, key: {:?}, subkeys: {:?}", self, record_key, subkeys);
            let subkeys = subkeys.unwrap_or_default();

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.cancel_watch_values(record_key, subkeys)).await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Inspects a DHT record for subkey state.
    /// This is useful for checking if you should push new subkeys to the network, or retrieve the current state of a record from the network
    /// to see what needs updating locally.
    ///
    /// * `key` is the record key to inspect. it must first be opened for reading or writing.
    /// * `subkeys`:
    ///   - None: specifies inspecting the entire range of subkeys.
    ///   - Some(range): is the the range of subkeys to inspect. The range must not exceed 512 discrete non-overlapping or adjacent subranges.
    ///                  If no range is specified, this is equivalent to watching the entire range of subkeys.
    /// * `scope` is what kind of range the inspection has:
    ///
    ///   - DHTReportScope::Local
    ///     Results will be only for a locally stored record.
    ///     Useful for seeing what subkeys you have locally and which ones have not been retrieved.
    ///
    ///   - DHTReportScope::SyncGet
    ///     Return the local sequence numbers and the network sequence numbers with GetValue fanout parameters.
    ///     Provides an independent view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if the local copy did not exist locally.
    ///     Useful for determining if the current local copy should be updated from the network.
    ///
    ///   - DHTReportScope::SyncSet
    ///     Return the local sequence numbers and the network sequence numbers with SetValue fanout parameters.
    ///     Provides an independent view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if the local copy did not exist locally.
    ///     Useful for determining if the unchanged local copy should be pushed to the network.
    ///
    ///   - DHTReportScope::UpdateGet
    ///     Return the local sequence numbers and the network sequence numbers with GetValue fanout parameters.
    ///     Provides an view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if a GetValue operation were being performed, including accepting newer values from the network.
    ///     Useful for determining which subkeys would change with a GetValue operation.
    ///
    ///   - DHTReportScope::UpdateSet
    ///     Return the local sequence numbers and the network sequence numbers with SetValue fanout parameters.
    ///     Provides an view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if a SetValue operation were being performed, including accepting newer values from the network.
    ///     This simulates a SetValue with the initial sequence number incremented by 1, like a real SetValue would when updating.
    ///     Useful for determine which subkeys would change with an SetValue operation.
    ///
    /// Returns a DHTRecordReport with the subkey ranges that were returned that overlapped the schema, and sequence numbers for each of the subkeys in the range.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn inspect_dht_record(
        &self,
        record_key: RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        scope: DHTReportScope,
    ) -> VeilidAPIResult<DHTRecordReport> {
        record_duration_fut(async {
            veilid_log!(self debug
                "RoutingContext::inspect_dht_record(self: {:?}, record_key: {:?}, subkeys: {:?}, scope: {:?})", self, record_key, subkeys, scope);
            let subkeys = subkeys.unwrap_or_default();

            self.api
                .core_context()?
                .storage_manager()
                .check_record_key(&record_key)?;

            let storage_manager = self.api.core_context()?.storage_manager();
            Box::pin(storage_manager.inspect_record(record_key, subkeys, scope)).await
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    ///////////////////////////////////
    /// Block Store

    #[cfg(feature = "unstable-blockstore")]
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn find_block(&self, _block_id: BlockId) -> VeilidAPIResult<Vec<u8>> {
        panic!("unimplemented");
    }

    #[cfg(feature = "unstable-blockstore")]
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret,)]
    pub async fn supply_block(&self, _block_id: BlockId) -> VeilidAPIResult<bool> {
        panic!("unimplemented");
    }
}
