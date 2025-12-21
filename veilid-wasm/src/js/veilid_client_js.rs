#![allow(non_snake_case)]
use super::*;
use veilid_crypto_js::VeilidCrypto;
use wasm_bindgen_derive::{try_from_js_array, try_from_js_option};

#[wasm_bindgen(typescript_custom_section)]
const IUPDATE_VEILID_FUNCTION: &'static str = r#"
export type UpdateVeilidFunction = (event: VeilidUpdate) => void;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Function, typescript_type = "UpdateVeilidFunction")]
    pub type UpdateVeilidFunction;
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[wasm_bindgen(js_name = veilidClient)]
pub struct VeilidClient {}

// Since this implementation doesn't contain a `new` fn that's marked as a constructor,
// and none of the member fns take a &self arg,
// this is just a namespace/class of static functions.
#[wasm_bindgen(js_class = veilidClient)]
impl VeilidClient {
    // --------------------------------
    // Constants
    // (written as getters since wasm_bindgen doesn't support export of const)
    // --------------------------------

    /// The ROUTE capability
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_ROUTE() -> JsValue {
        crate::VEILID_CAPABILITY_ROUTE.into()
    }

    /// The TUNL capability
    #[cfg(feature = "unstable-tunnels")]
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_TUNNEL() -> JsValue {
        crate::VEILID_CAPABILITY_TUNNEL.into()
    }

    /// The SGNL capability
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_SIGNAL() -> JsValue {
        crate::VEILID_CAPABILITY_SIGNAL.into()
    }

    /// The RLAY capability
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_RELAY() -> JsValue {
        crate::VEILID_CAPABILITY_RELAY.into()
    }

    /// The DIAL capability
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_VALIDATE_DIAL_INFO() -> JsValue {
        crate::VEILID_CAPABILITY_VALIDATE_DIAL_INFO.into()
    }

    /// The DHTV capability
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_DHT() -> JsValue {
        crate::VEILID_CAPABILITY_DHT.into()
    }

    /// The APPM capability
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_APPMESSAGE() -> JsValue {
        crate::VEILID_CAPABILITY_APPMESSAGE.into()
    }

    /// The BLOC capability
    #[cfg(feature = "unstable-blockstore")]
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability")]
    #[must_use]
    pub fn VEILID_CAPABILITY_BLOCKSTORE() -> JsValue {
        crate::VEILID_CAPABILITY_BLOCKSTORE.into()
    }

    /// All distance metric capabilites of this version of Veilid
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability[]")]
    #[must_use]
    pub fn DISTANCE_METRIC_CAPABILITIES() -> JsValue {
        js_sys::Array::from_iter(
            crate::DISTANCE_METRIC_CAPABILITIES
                .iter()
                .map(|x| JsValue::from(x.to_string())),
        )
        .into()
    }

    /// All connectivity capabilites of this version of Veilid
    #[wasm_bindgen(getter, unchecked_return_type = "VeilidCapability[]")]
    #[must_use]
    pub fn CONNECTIVITY_CAPABILITIES() -> JsValue {
        js_sys::Array::from_iter(
            crate::CONNECTIVITY_CAPABILITIES
                .iter()
                .map(|x| JsValue::from(x.to_string())),
        )
        .into()
    }

    ///////////////////////////////////////////////////////////////////////////////////////////

    #[allow(clippy::unused_async)]
    pub async fn initializeCore(platformConfig: VeilidWASMConfig) {
        if INITIALIZED.swap(true, Ordering::AcqRel) {
            return;
        }
        console_error_panic_hook::set_once();

        // Set up subscriber and layers
        let subscriber = Registry::default();
        let mut layers = Vec::new();
        let mut filters = (*FILTERS).borrow_mut();

        // Performance logger
        if platformConfig.logging.performance.enabled {
            let filter = VeilidLayerFilter::new(
                platformConfig.logging.performance.level,
                &platformConfig.logging.performance.ignore_log_targets,
                None,
            );
            let layer = WASMLayer::new(
                WASMLayerConfig::new()
                    .with_report_logs_in_timings(platformConfig.logging.performance.logs_in_timings)
                    .with_console_config(match platformConfig.logging.performance.logs_in_console {
                        VeilidWASMConfigLoggingLogsInConsole::Off => ConsoleConfig::NoReporting,
                        VeilidWASMConfigLoggingLogsInConsole::NoColor => {
                            ConsoleConfig::ReportWithoutConsoleColor
                        }
                        VeilidWASMConfigLoggingLogsInConsole::Color => {
                            ConsoleConfig::ReportWithConsoleColor
                        }
                    })
                    .with_field_filter(Some(Arc::new(|k| k != VEILID_LOG_KEY_FIELD))),
            )
            .with_filter(filter.clone());
            filters.insert("performance", filter);
            layers.push(layer.boxed());
        };

        // API logger
        if platformConfig.logging.api.enabled {
            let filter = VeilidLayerFilter::new(
                platformConfig.logging.api.level,
                &platformConfig.logging.api.ignore_log_targets,
                None,
            );
            let layer = ApiTracingLayer::init().with_filter(filter.clone());
            filters.insert("api", filter);
            layers.push(layer.boxed());
        }

        let subscriber = subscriber.with(layers);
        subscriber
            .try_init()
            .map_err(|e| format!("failed to initialize logging: {}", e))
            .expect("failed to initalize WASM platform");
    }

    /// Initialize a Veilid node, with the configuration in JSON format
    ///
    /// Must be called only once at the start of an application
    ///
    /// @param {UpdateVeilidFunction} updateCallbackJS - called when internal state of the Veilid node changes, for example, when app-level messages are received, when private routes die and need to be reallocated, or when routing table states change
    /// @param {VeilidConfig} config - the configuration object to use for the instance
    pub async fn startupCore(
        updateCallbackJS: UpdateVeilidFunction,
        config: VeilidConfig,
    ) -> VeilidAPIResult<()> {
        let update_callback_js = SendWrapper::new(updateCallbackJS);
        let update_callback = Arc::new(move |update: VeilidUpdate| {
            let _ret = match Function::call1(
                &update_callback_js,
                &JsValue::UNDEFINED,
                &to_jsvalue(update),
            ) {
                Ok(v) => v,
                Err(e) => {
                    console_log(&format!("calling update callback failed: {:?}", e));
                    return;
                }
            };
        });

        if VEILID_API.borrow().is_some() {
            return VeilidAPIResult::Err(VeilidAPIError::AlreadyInitialized);
        }

        let veilid_api = api_startup(update_callback, config).await?;
        VEILID_API.replace(Some(veilid_api));
        Ok(())
    }

    // TODO: can we refine the TS type of `layer`?
    pub fn changeLogLevel(layer: String, logLevel: VeilidConfigLogLevel) {
        let layer = if layer == "all" { "".to_owned() } else { layer };
        let filters = (*FILTERS).borrow();
        if layer.is_empty() {
            // Change all layers
            for f in filters.values() {
                f.set_max_level(logLevel);
            }
        } else {
            // Change a specific layer
            if let Some(f) = filters.get(layer.as_str()) {
                f.set_max_level(logLevel);
            }
        }
    }

    // TODO: can we refine the TS type of `layer`?
    pub fn changeLogIgnore(layer: String, changes: Vec<String>) {
        let layer = if layer == "all" { "".to_owned() } else { layer };
        let filters = (*FILTERS).borrow();
        if layer.is_empty() {
            // Change all layers
            for f in filters.values() {
                let mut ignore_list = f.ignore_list();
                VeilidLayerFilter::apply_ignore_change_list(&mut ignore_list, &changes);
                f.set_ignore_list(Some(ignore_list));
            }
        } else {
            // Change a specific layer
            if let Some(f) = filters.get(layer.as_str()) {
                let mut ignore_list = f.ignore_list();
                VeilidLayerFilter::apply_ignore_change_list(&mut ignore_list, &changes);
                f.set_ignore_list(Some(ignore_list));
            }
        }
    }
    /// Shut down Veilid and terminate the API.
    pub async fn shutdownCore() -> VeilidAPIResult<()> {
        let veilid_api = take_veilid_api()?;
        veilid_api.shutdown().await;
        Ok(())
    }

    /// Check if Veilid is shutdown.
    pub fn isShutdown() -> VeilidAPIResult<bool> {
        let veilid_api = get_veilid_api();
        if let Err(VeilidAPIError::NotInitialized) = veilid_api {
            return Ok(true);
        }
        let veilid_api = veilid_api.unwrap();
        let is_shutdown = veilid_api.is_shutdown();
        Ok(is_shutdown)
    }

    /// Get a full copy of the current state of Veilid.
    pub async fn getState() -> VeilidAPIResult<VeilidState> {
        let veilid_api = get_veilid_api()?;
        veilid_api.get_state().await
    }

    /// Connect to the network.
    pub async fn attach() -> VeilidAPIResult<()> {
        let veilid_api = get_veilid_api()?;
        veilid_api.attach().await
    }

    /// Disconnect from the network.
    pub async fn detach() -> VeilidAPIResult<()> {
        let veilid_api = get_veilid_api()?;
        veilid_api.detach().await
    }

    /// Get a cryptosystem by its kind
    pub fn getCrypto(
        #[wasm_bindgen(unchecked_param_type = "CryptoKind")] kind: JsValue,
    ) -> VeilidAPIResult<VeilidCrypto> {
        let kind = CryptoKind::from_js(kind).map_err(VeilidAPIError::generic)?;
        let veilid_api = get_veilid_api()?;
        if veilid_api.crypto()?.get(kind).is_none() {
            apibail_invalid_argument!("get_crypto", "kind", kind);
        }
        Ok(VeilidCrypto { kind })
    }

    /// Verify multiple signatures with multiple cryptosystems
    pub fn verifySignatures(
        #[wasm_bindgen(unchecked_param_type = "PublicKey[]")] publicKeys: JsValue,
        data: Box<[u8]>,
        #[wasm_bindgen(unchecked_param_type = "Signature[]")] signatures: JsValue,
    ) -> VeilidAPIResult<Option<Vec<PublicKey>>> {
        let public_keys =
            try_from_js_array::<PublicKey>(publicKeys).map_err(VeilidAPIError::generic)?;
        let signatures =
            try_from_js_array::<Signature>(signatures).map_err(VeilidAPIError::generic)?;

        let veilid_api = get_veilid_api()?;
        let crypto = veilid_api.crypto()?;
        let out = crypto.verify_signatures(&public_keys, &data, &signatures)?;
        Ok(out.map(|v| v.iter().cloned().collect()))
    }

    /// Generate multiple signatures with multiple cryptosystems
    pub fn generateSignatures(
        data: Box<[u8]>,
        #[wasm_bindgen(unchecked_param_type = "KeyPair[]")] keyPairs: JsValue,
    ) -> VeilidAPIResult<Vec<Signature>> {
        let key_pairs = try_from_js_array::<KeyPair>(keyPairs).map_err(VeilidAPIError::generic)?;
        let veilid_api = get_veilid_api()?;
        let crypto = veilid_api.crypto()?;
        crypto.generate_signatures(&data, &key_pairs, |_k, s| s)
    }

    /// Create a new MemberId for use with in creating `DHTSchema`s.
    pub fn generateMemberId(writer_key: &PublicKey) -> VeilidAPIResult<MemberId> {
        let veilid_api = get_veilid_api()?;
        veilid_api.generate_member_id(writer_key)
    }

    /// Start a transaction on a set of DHT records
    /// Record keys must have been opened via a routing context already when passed to this function
    /// Options can be specified that supply a default signing keypair for records that are not opened for writing
    pub async fn transactDHTRecords(
        #[wasm_bindgen(unchecked_param_type = "RecordKey[]")] recordKeys: JsValue,
        options: Option<ts::TransactDHTRecordsOptions>,
    ) -> VeilidAPIResult<VeilidDHTTransaction> {
        let record_keys =
            try_from_js_array::<RecordKey>(recordKeys).map_err(VeilidAPIError::generic)?;

        let veilid_api = get_veilid_api()?;

        let dht_transaction = veilid_api
            .transact_dht_records(
                record_keys,
                match options {
                    Some(o) => Some(o.try_into()?),
                    None => None,
                },
            )
            .await?;

        Ok(VeilidDHTTransaction {
            inner_transaction: Some(dht_transaction),
        })
    }

    /// Deterministicly builds the record key for a given schema and owner public key.
    /// The crypto kind of the record key will be that of the `owner` public key
    #[allow(clippy::unused_async)]
    pub async fn getDHTRecordKey(
        schema: ts::DHTSchema,
        owner: &PublicKey,
        encryptionKey: Option<TypeStubSharedSecret>,
    ) -> VeilidAPIResult<RecordKey> {
        let schema = schema.try_into()?;

        let encryption_key = match encryptionKey {
            Some(encryption_key) => try_from_js_option::<SharedSecret>(encryption_key)
                .map_err(VeilidAPIError::generic)?,
            None => None,
        };

        let veilid_api = get_veilid_api()?;

        veilid_api.get_dht_record_key(schema, owner.clone(), encryption_key)
    }

    /// Allocate a new private route set with default cryptography and network options.
    /// Returns a route id and a publishable 'blob' with the route encrypted with each crypto kind.
    /// Those nodes importing the blob will have their choice of which crypto kind to use.
    ///
    /// Returns a route id and 'blob' that can be published over some means (DHT or otherwise) to be imported by another Veilid node.
    pub async fn newPrivateRoute() -> VeilidAPIResult<RouteBlob> {
        let veilid_api = get_veilid_api()?;

        veilid_api.new_private_route().await
    }

    /// Import a private route blob as a remote private route.
    ///
    /// Returns a route id that can be used to send private messages to the node creating this route.
    #[allow(clippy::boxed_local)]
    pub fn importRemotePrivateRoute(&self, blob: Box<[u8]>) -> VeilidAPIResult<RouteId> {
        let veilid_api = get_veilid_api()?;
        veilid_api.import_remote_private_route(blob.to_vec())
    }

    /// Allocate a new private route and specify a specific cryptosystem, stability and sequencing preference.
    /// Returns a route id and a publishable 'blob' with the route encrypted with each crypto kind.
    /// Those nodes importing the blob will have their choice of which crypto kind to use.
    ///
    /// Returns a route id and 'blob' that can be published over some means (DHT or otherwise) to be imported by another Veilid node.
    pub async fn newCustomPrivateRoute(
        stability: Stability,
        sequencing: Sequencing,
    ) -> VeilidAPIResult<RouteBlob> {
        let veilid_api = get_veilid_api()?;

        veilid_api
            .new_custom_private_route(&VALID_CRYPTO_KINDS, stability, sequencing)
            .await
    }

    /// Release either a locally allocated or remotely imported private route.
    ///
    /// This will deactivate the route and free its resources and it can no longer be sent to or received from.
    pub fn releasePrivateRoute(route_id: &RouteId) -> VeilidAPIResult<()> {
        let veilid_api = get_veilid_api()?;
        veilid_api.release_private_route(route_id.clone())
    }

    /// Respond to an AppCall received over a VeilidUpdate::AppCall.
    ///
    /// * `call_id` - specifies which call to reply to, and it comes from a VeilidUpdate::AppCall, specifically the VeilidAppCall::id() value.
    /// * `message` - is an answer blob to be returned by the remote node's RoutingContext::app_call() function, and may be up to 32768 bytes
    pub async fn appCallReply(callId: String, message: Box<[u8]>) -> VeilidAPIResult<()> {
        let message = message.to_vec();
        let call_id = match callId.parse() {
            Ok(v) => v,
            Err(e) => {
                return VeilidAPIResult::Err(VeilidAPIError::invalid_argument(e, "call_id", callId))
            }
        };
        let veilid_api = get_veilid_api()?;
        veilid_api.app_call_reply(call_id, message).await
    }

    /// Get the current timestamp, in string format
    #[must_use]
    pub fn now() -> String {
        Timestamp::now().as_u64().to_string()
    }

    /// Execute an 'internal debug command'.
    pub async fn debug(command: String) -> VeilidAPIResult<String> {
        let veilid_api = get_veilid_api()?;
        veilid_api.debug(command).await
    }

    /// Return the cargo package version of veilid-core, in object format.
    #[must_use]
    pub fn version() -> VeilidVersion {
        let (major, minor, patch) = veilid_version();
        super::VeilidVersion {
            major,
            minor,
            patch,
        }
    }

    /// Return the features that were enabled when veilid-core was built.
    #[must_use]
    pub fn features() -> Vec<String> {
        veilid_features()
    }

    /// Return the cargo package version of veilid-core, in string format.
    #[must_use]
    pub fn versionString() -> String {
        veilid_version_string()
    }

    /// Return the default veilid configuration, in string format
    pub fn defaultConfig() -> VeilidConfig {
        VeilidConfig::default()
    }
}

/////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
pub struct VeilidDHTTransaction {
    inner_transaction: Option<DHTTransaction>,
}

#[wasm_bindgen]
impl VeilidDHTTransaction {
    fn getTransaction(&self) -> VeilidAPIResult<DHTTransaction> {
        let Some(transaction) = &self.inner_transaction else {
            return VeilidAPIResult::Err(veilid_core::VeilidAPIError::generic(
                "Unable to getTransaction instance. inner_transaction is None.",
            ));
        };
        VeilidAPIResult::Ok(transaction.clone())
    }

    /// Commit the transaction. Performs all actions atomically.
    pub async fn commit(&self) -> VeilidAPIResult<()> {
        let transaction = self.getTransaction()?;
        transaction.commit().await
    }

    /// Rollback the transaction. Does nothing to the DHT.
    pub async fn rollback(&self) -> VeilidAPIResult<()> {
        let transaction = self.getTransaction()?;
        transaction.rollback().await
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
    pub async fn set(
        &self,
        recordKey: &RecordKey,
        subkey: ValueSubkey,
        data: Vec<u8>,
        options: Option<ts::DHTTransactionSetValueOptions>,
    ) -> VeilidAPIResult<Option<ts::ValueData>> {
        let transaction = self.getTransaction()?;

        transaction
            .set(
                recordKey.clone(),
                subkey,
                data,
                match options {
                    Some(o) => Some(o.try_into()?),
                    None => None,
                },
            )
            .await
            .map(|x| x.map(|y| y.into()))
    }

    /// Perform a get_dht_value operation inside the transaction
    ///
    /// * Will fail if performed offline
    /// * Will pull the latest value from the network, will fail if the local value is newer
    /// * Will fail if existing offline writes exist for this record key
    ///
    /// Returns `None` if the value subkey has not yet been set.
    /// Returns `Some(data)` if the value subkey has valid data.
    pub async fn get(
        &self,
        recordKey: &RecordKey,
        subkey: ValueSubkey,
    ) -> VeilidAPIResult<Option<ts::ValueData>> {
        let transaction = self.getTransaction()?;

        transaction
            .get(recordKey.clone(), subkey)
            .await
            .map(|x| x.map(|y| y.into()))
    }

    /// Perform a inspect_dht_record operation inside the transaction
    ///
    /// * Does not perform any network activity, as the transaction state keeps all of the required information after the begin
    ///
    /// For information on arguments, see [RoutingContext::inspect_dht_record]
    ///
    /// Returns a DHTRecordReport with the subkey ranges that were returned that overlapped the schema, and sequence numbers for each of the subkeys in the range.
    pub async fn inspect(
        &self,
        recordKey: &RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        scope: Option<DHTReportScope>,
    ) -> VeilidAPIResult<DHTRecordReport> {
        let scope = scope.unwrap_or_default();

        let transaction = self.getTransaction()?;

        transaction.inspect(recordKey.clone(), subkeys, scope).await
    }
}
