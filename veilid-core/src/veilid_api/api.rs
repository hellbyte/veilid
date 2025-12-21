use super::*;

impl_veilid_log_facility!("veilid_api");

/////////////////////////////////////////////////////////////////////////////////////////////////////

pub(super) struct VeilidAPIInner {
    context: Option<VeilidCoreContext>,
    pub(super) debug_cache: DebugCache,
}

impl fmt::Debug for VeilidAPIInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VeilidAPIInner")
    }
}

impl Drop for VeilidAPIInner {
    fn drop(&mut self) {
        if let Some(context) = self.context.take() {
            spawn_detached("api shutdown", api_shutdown(context));
        }
    }
}

/// The primary developer entrypoint into `veilid-core` functionality.
///
/// From [VeilidAPI] one can access various components:
///
/// * [VeilidConfig] - The Veilid configuration specified at startup time.
/// * [Crypto] - The available set of cryptosystems provided by Veilid.
/// * [TableStore] - The Veilid table-based encrypted persistent key-value store.
/// * [ProtectedStore] - The Veilid abstract of the device's low-level 'protected secret storage'.
/// * [VeilidState] - The current state of the Veilid node this API accesses.
/// * [RoutingContext] - Communication methods between Veilid nodes and private routes.
/// * Attach and detach from the network.
/// * Create and import private routes.
/// * Reply to `AppCall` RPCs.
#[derive(Clone, Debug)]
#[must_use]
pub struct VeilidAPI {
    inner: Arc<Mutex<VeilidAPIInner>>,
}

impl VeilidAPI {
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = context.log_key()), skip_all)]
    pub(crate) fn new(context: VeilidCoreContext) -> Self {
        veilid_log!(context debug "VeilidAPI::new()");
        record_duration(|| Self {
            inner: Arc::new(Mutex::new(VeilidAPIInner {
                context: Some(context),
                debug_cache: DebugCache {
                    imported_routes: Vec::new(),
                    opened_record_contexts: once_cell::sync::Lazy::new(
                        hashlink::LinkedHashMap::new,
                    ),
                },
            })),
        })
    }

    /// Shut down Veilid and terminate the API.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip_all)]
    pub async fn shutdown(self) {
        record_duration_fut(async {
            veilid_log!(self debug "VeilidAPI::shutdown()");
            let context = { self.inner.lock().context.take() };
            if let Some(context) = context {
                api_shutdown(context).await;
            }
        })
        .await
    }

    /// Check to see if Veilid is already shut down.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.inner.lock().context.is_none()
    }

    ////////////////////////////////////////////////////////////////
    // Public Accessors

    /// Access the configuration that Veilid was initialized with.
    pub fn config(&self) -> VeilidAPIResult<Arc<VeilidConfig>> {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return Err(VeilidAPIError::NotInitialized);
        };
        Ok(context.registry().config())
    }

    /// Get the cryptosystem component.
    pub fn crypto(&self) -> VeilidAPIResult<VeilidComponentGuard<'_, Crypto>> {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return Err(VeilidAPIError::NotInitialized);
        };
        context
            .registry()
            .lookup::<Crypto>()
            .ok_or(VeilidAPIError::NotInitialized)
    }

    /// Get the TableStore component.
    pub fn table_store(&self) -> VeilidAPIResult<VeilidComponentGuard<'_, TableStore>> {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return Err(VeilidAPIError::NotInitialized);
        };
        context
            .registry()
            .lookup::<TableStore>()
            .ok_or(VeilidAPIError::NotInitialized)
    }

    /// Get the ProtectedStore component.
    pub fn protected_store(&self) -> VeilidAPIResult<VeilidComponentGuard<'_, ProtectedStore>> {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return Err(VeilidAPIError::NotInitialized);
        };
        context
            .registry()
            .lookup::<ProtectedStore>()
            .ok_or(VeilidAPIError::NotInitialized)
    }

    /// Get the BlockStore component.
    #[cfg(feature = "unstable-blockstore")]
    pub fn block_store(&self) -> VeilidAPIResult<VeilidComponentGuard<'_, BlockStore>> {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return Err(VeilidAPIError::NotInitialized);
        };
        context
            .registry()
            .lookup::<BlockStore>()
            .ok_or(VeilidAPIError::NotInitialized)
    }

    ////////////////////////////////////////////////////////////////
    // Internal Accessors

    pub(crate) fn core_context(&self) -> VeilidAPIResult<VeilidCoreContext> {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return Err(VeilidAPIError::NotInitialized);
        };
        Ok(context.clone())
    }

    pub(crate) fn with_debug_cache<R, F: FnOnce(&mut DebugCache) -> R>(&self, callback: F) -> R {
        let mut inner = self.inner.lock();
        callback(&mut inner.debug_cache)
    }

    #[must_use]
    pub(crate) fn log_key(&self) -> &str {
        let inner = self.inner.lock();
        let Some(context) = &inner.context else {
            return "";
        };
        context.log_key()
    }

    ////////////////////////////////////////////////////////////////
    // Attach/Detach

    /// Get a full copy of the current state of Veilid.
    #[expect(clippy::unused_async)]
    pub async fn get_state(&self) -> VeilidAPIResult<VeilidState> {
        let attachment_manager = self.core_context()?.attachment_manager();
        let network_manager = attachment_manager.network_manager();
        let config = self.config()?;

        let attachment = attachment_manager.get_veilid_state();
        let network = network_manager.get_veilid_state();

        Ok(VeilidState {
            attachment,
            network,
            config: Box::new(VeilidStateConfig {
                config: config.as_ref().clone(),
            }),
        })
    }

    /// Connect to the network.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip_all, ret)]
    pub async fn attach(&self) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            veilid_log!(self debug
                "VeilidAPI::attach()");
            let attachment_manager = self.core_context()?.attachment_manager();
            if !Box::pin(attachment_manager.attach()).await {
                apibail_generic!("Already attached");
            }
            Ok(())
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Disconnect from the network.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip_all, ret)]
    pub async fn detach(&self) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            veilid_log!(self debug
            "VeilidAPI::detach()");

            let attachment_manager = self.core_context()?.attachment_manager();
            if !Box::pin(attachment_manager.detach()).await {
                apibail_generic!("Already detached");
            }
            Ok(())
        })
        .await
        .inspect_err(log_veilid_api_error!(self))
    }

    ////////////////////////////////////////////////////////////////
    // Routing Context

    /// Get a new `RoutingContext` object to use to send messages over the Veilid network with default safety, sequencing, and stability parameters.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip_all, ret)]
    pub fn routing_context(&self) -> VeilidAPIResult<RoutingContext> {
        record_duration(|| {
            veilid_log!(self debug
                "VeilidAPI::routing_context()");

            RoutingContext::try_new(self.clone())
        })
        .inspect_err(log_veilid_api_error!(self))
    }

    ////////////////////////////////////////////////////////////////
    // Non-RoutingContext DHT Operations

    /// Deterministicly builds the record key for a given schema and owner public key.
    /// The crypto kind of the record key will be that of the `owner` public key
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub fn get_dht_record_key(
        &self,
        schema: DHTSchema,
        owner_key: PublicKey,
        encryption_key: Option<SharedSecret>,
    ) -> VeilidAPIResult<RecordKey> {
        record_duration(|| {
            veilid_log!(self debug
            "RoutingContext::get_dht_record_key(self: {:?} schema: {:?}, owner_key: {:?}, encryption_key: {:?}", self, schema, owner_key, encryption_key);
            schema.validate()?;

            self.crypto()?.check_public_key(&owner_key)?;
            if let Some(encryption_key) = encryption_key.as_ref() {
                self.crypto()?.check_shared_secret(encryption_key)?;
            }

            let storage_manager = self.core_context()?.storage_manager();
            storage_manager.get_record_key(schema, &owner_key, encryption_key)
        }).inspect_err(log_veilid_api_error!(self))
    }

    /// Create a new MemberId for use with in creating `DHTSchema`s.
    #[instrument(target = "veilid_api", level = "debug", skip(self), fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub fn generate_member_id(&self, writer_key: &PublicKey) -> VeilidAPIResult<MemberId> {
        record_duration(|| {
            veilid_log!(self debug "VeilidAPI::generate_member_id(writer_key: {:?}", writer_key);

            self.crypto()?.check_public_key(writer_key)?;

            let storage_manager = self.core_context()?.storage_manager();
            storage_manager.generate_member_id(writer_key)
        })
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Start a transaction on a set of DHT records
    /// Record keys must have been opened via a routing context already when passed to this function
    /// The maximum number of records per transaction is currently 32.
    /// Options can be specified that supply a default signing keypair for records that are not opened for writing
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), ret)]
    pub async fn transact_dht_records(
        &self,
        record_keys: Vec<RecordKey>,
        options: Option<TransactDHTRecordsOptions>,
    ) -> VeilidAPIResult<DHTTransaction> {
        record_duration_fut(async {
            veilid_log!(self debug
                "VeilidAPI::transact_dht_records(self: {:?}, record_keys: {:?}, options: {:?})", self, record_keys, options);

            let storage_manager = self.core_context()?.storage_manager();

            for record_key in &record_keys {
                storage_manager.check_record_key(record_key)?;
            }

            let handle = Box::pin(storage_manager.begin_transaction(record_keys, options)).await?;

            DHTTransaction::new(self.clone(), handle.clone())
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    ////////////////////////////////////////////////////////////////
    // Private route allocation

    /// Allocate a new private route set with default cryptography and network options.
    /// Default settings are for [Stability::Reliable] and [Sequencing::PreferOrdered].
    /// Returns a route id and a publishable 'blob' with the route encrypted with each crypto kind.
    /// Those nodes importing the blob will have their choice of which crypto kind to use.
    ///
    /// Returns a route id and 'blob' that can be published over some means (DHT or otherwise) to be
    /// imported by another Veilid node.
    pub async fn new_private_route(&self) -> VeilidAPIResult<RouteBlob> {
        record_duration_fut(async {
            Box::pin(self.new_custom_private_route(
                &VALID_CRYPTO_KINDS,
                Stability::Reliable,
                Sequencing::PreferOrdered,
            ))
            .await
        })
        .await
    }

    /// Allocate a new private route and specify a specific cryptosystem, stability and sequencing preference.
    /// Faster connections may be possible with [Stability::LowLatency], and [Sequencing::NoPreference] at the
    /// expense of some loss of messages.
    /// Returns a route id and a publishable 'blob' with the route encrypted with each crypto kind.
    /// Those nodes importing the blob will have their choice of which crypto kind to use.
    ///
    /// Returns a route id and 'blob' that can be published over some means (DHT or otherwise) to be
    /// imported by another Veilid node.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub async fn new_custom_private_route(
        &self,
        crypto_kinds: &[CryptoKind],
        stability: Stability,
        sequencing: Sequencing,
    ) -> VeilidAPIResult<RouteBlob> {
        record_duration_fut(async {
            veilid_log!(self debug
                "VeilidAPI::new_custom_private_route(crypto_kinds: {:?}, stability: {:?}, sequencing: {:?})",
                crypto_kinds,
                stability,
                sequencing);

            for kind in crypto_kinds {
                Crypto::validate_crypto_kind(*kind)?;
            }

            let default_route_hop_count: usize =
                self.config()?.network.rpc.default_route_hop_count.into();

            let safety_spec = SafetySpec {
                preferred_route: None,
                hop_count: default_route_hop_count,
                stability,
                sequencing,
            };

            let routing_table = self.core_context()?.routing_table();
            let rss = routing_table.route_spec_store();
            let route_id =
                rss.allocate_route(crypto_kinds, &safety_spec, DirectionSet::all(), &[], false)?;
            match Box::pin(rss.test_route(route_id.clone())).await? {
                Some(true) => {
                    // route tested okay
                }
                Some(false) => {
                    rss.release_route(route_id.clone());
                    apibail_generic!("allocated route failed to test");
                }
                None => {
                    rss.release_route(route_id.clone());
                    apibail_generic!("allocated route could not be tested");
                }
            }
            let private_routes = rss.assemble_private_route_set(&route_id, Some(true))?;
            let blob = match RouteSpecStore::private_routes_to_blob(&private_routes) {
                Ok(v) => v,
                Err(e) => {
                    rss.release_route(route_id);
                    return Err(e);
                }
            };

            rss.mark_route_published(&route_id, true)?;

            Ok(RouteBlob { route_id, blob })
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    /// Import a private route blob as a remote private route.
    ///
    /// Returns a route id that can be used to send private messages to the node creating this route.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub fn import_remote_private_route(&self, blob: Vec<u8>) -> VeilidAPIResult<RouteId> {
        record_duration(|| {
            veilid_log!(self debug
                "VeilidAPI::import_remote_private_route(blob: {:?})", blob);
            let routing_table = self.core_context()?.routing_table();
            let rss = routing_table.route_spec_store();
            rss.import_remote_route_blob(blob)
        })
        .inspect_err(log_veilid_api_error!(self))
    }

    /// Release either a locally allocated or remotely imported private route.
    ///
    /// This will deactivate the route and free its resources and it can no longer be sent to
    /// or received from.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub fn release_private_route(&self, route_id: RouteId) -> VeilidAPIResult<()> {
        record_duration(|| {
            veilid_log!(self debug
                "VeilidAPI::release_private_route(route_id: {:?})", route_id);

            let routing_table = self.core_context()?.routing_table();
            routing_table.check_route_id(&route_id)?;

            let rss = routing_table.route_spec_store();
            if !rss.release_route(route_id.clone()) {
                apibail_invalid_argument!("release_private_route", "key", route_id);
            }
            Ok(())
        })
        .inspect_err(log_veilid_api_error!(self))
    }

    ////////////////////////////////////////////////////////////////
    // App Calls

    /// Respond to an AppCall received over a [VeilidUpdate::AppCall].
    ///
    /// * `call_id` - specifies which call to reply to, and it comes from a [VeilidUpdate::AppCall], specifically the [VeilidAppCall::id()] value.
    /// * `message` - is an answer blob to be returned by the remote node's [RoutingContext::app_call()] function, and may be up to 32768 bytes.
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub async fn app_call_reply(
        &self,
        call_id: OperationId,
        message: Vec<u8>,
    ) -> VeilidAPIResult<()> {
        record_duration_fut(async {
            veilid_log!(self debug
                "VeilidAPI::app_call_reply(call_id: {:?}, message_len: {})", call_id, message.len());
            veilid_log!(self trace "message: {:?}", message);

            let rpc_processor = self.core_context()?.rpc_processor();
            rpc_processor
                .app_call_reply(call_id, message)
                .map_err(|e| e.into())
        }).await.inspect_err(log_veilid_api_error!(self))
    }

    ////////////////////////////////////////////////////////////////
    // Tunnel Building

    #[cfg(feature = "unstable-tunnels")]
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub async fn start_tunnel(
        &self,
        _endpoint_mode: TunnelMode,
        _depth: u8,
    ) -> VeilidAPIResult<PartialTunnel> {
        panic!("unimplemented");
    }

    #[cfg(feature = "unstable-tunnels")]
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub async fn complete_tunnel(
        &self,
        _endpoint_mode: TunnelMode,
        _depth: u8,
        _partial_tunnel: PartialTunnel,
    ) -> VeilidAPIResult<FullTunnel> {
        panic!("unimplemented");
    }

    #[cfg(feature = "unstable-tunnels")]
    #[instrument(target = "veilid_api", level = "debug", fields(duration, __VEILID_LOG_KEY = self.log_key()), skip(self), ret)]
    pub async fn cancel_tunnel(&self, _tunnel_id: TunnelId) -> VeilidAPIResult<bool> {
        panic!("unimplemented");
    }
}
