#![allow(non_snake_case)]
use super::*;
use wasm_bindgen_derive::try_from_js_option;

#[wasm_bindgen()]
pub struct VeilidRoutingContext {
    inner_routing_context: RoutingContext,
}

#[wasm_bindgen()]
impl VeilidRoutingContext {
    /// Create a new VeilidRoutingContext, without any privacy or sequencing settings.
    #[wasm_bindgen(constructor)]
    pub fn new() -> VeilidAPIResult<VeilidRoutingContext> {
        let veilid_api = get_veilid_api()?;
        VeilidAPIResult::Ok(VeilidRoutingContext {
            inner_routing_context: veilid_api.routing_context()?,
        })
    }

    /// Same as `new VeilidRoutingContext()` except easier to chain.
    pub fn create() -> VeilidAPIResult<VeilidRoutingContext> {
        VeilidRoutingContext::new()
    }

    // --------------------------------
    // Methods
    // --------------------------------

    fn getRoutingContext(&self) -> VeilidAPIResult<RoutingContext> {
        Ok(self.inner_routing_context.clone())
    }

    /// Turn on sender privacy, enabling the use of safety routes. This is the default and
    /// calling this function is only necessary if you have previously disable safety or used other parameters.
    /// Returns a new instance of VeilidRoutingContext - does not mutate.
    ///
    /// Default values for hop count, stability and sequencing preferences are used.
    ///
    /// * Hop count default is dependent on config, but is set to 1 extra hop.
    /// * Stability default is to choose 'low latency' routes, preferring them over long-term reliability.
    /// * Sequencing default is to have no preference for ordered vs unordered message delivery
    ///
    /// To customize the safety selection in use, use [VeilidRoutingContext::withSafety].
    pub fn withDefaultSafety(&self) -> VeilidAPIResult<VeilidRoutingContext> {
        let routing_context = self.getRoutingContext()?;
        Ok(VeilidRoutingContext {
            inner_routing_context: routing_context.with_default_safety()?,
        })
    }

    /// Use a custom [SafetySelection]. Can be used to disable safety via [SafetySelection::Unsafe]
    /// Returns a new instance of VeilidRoutingContext - does not mutate.
    pub fn withSafety(
        &self,
        safety_selection: SafetySelection,
    ) -> VeilidAPIResult<VeilidRoutingContext> {
        let routing_context = self.getRoutingContext()?;
        Ok(VeilidRoutingContext {
            inner_routing_context: routing_context.with_safety(safety_selection)?,
        })
    }

    /// Use a specified `Sequencing` preference.
    /// Returns a new instance of VeilidRoutingContext - does not mutate.
    pub fn withSequencing(&self, sequencing: Sequencing) -> VeilidAPIResult<VeilidRoutingContext> {
        let routing_context = self.getRoutingContext()?;
        Ok(VeilidRoutingContext {
            inner_routing_context: routing_context.with_sequencing(sequencing),
        })
    }

    /// Get the safety selection in use on this routing context
    /// @returns the SafetySelection currently in use if successful.
    pub fn safety(&self) -> VeilidAPIResult<SafetySelection> {
        let routing_context = self.getRoutingContext()?;

        let safety_selection = routing_context.safety();
        Ok(safety_selection)
    }
    /// App-level unidirectional message that does not expect any value to be returned.
    ///
    /// Veilid apps may use this for arbitrary message passing.
    ///
    /// @param {string} target - a private route id, or in 'footgun' mode, a direct node id
    /// @param {string} message - an arbitrary message blob of up to `32768` bytes.
    #[wasm_bindgen(skip_jsdoc)]
    pub async fn appMessage(&self, target: Target, message: Box<[u8]>) -> VeilidAPIResult<()> {
        let routing_context = self.getRoutingContext()?;
        let message = message.into_vec();
        routing_context.app_message(target, message).await
    }

    /// App-level bidirectional call that expects a response to be returned.
    ///
    /// Veilid apps may use this for arbitrary message passing.
    ///
    /// @param {string} target - a private route id, or in 'footgun' mode, a direct node id
    /// @param {Uint8Array} message - an arbitrary message blob of up to `32768` bytes.
    /// @returns {Uint8Array} an answer blob of up to `32768` bytes.
    #[wasm_bindgen(skip_jsdoc)]
    pub async fn appCall(&self, target: Target, request: Box<[u8]>) -> VeilidAPIResult<Uint8Array> {
        let request: Vec<u8> = request.into_vec();
        let routing_context = self.getRoutingContext()?;

        let answer = routing_context.app_call(target, request).await?;
        let answer = Uint8Array::from(answer.as_slice());
        Ok(answer)
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
    pub async fn createDHTRecord(
        &self,
        #[wasm_bindgen(unchecked_param_type = "CryptoKind")] kind: JsValue,
        schema: ts::DHTSchema,
        owner: Option<TypeStubKeyPair>,
    ) -> VeilidAPIResult<ts::DHTRecordDescriptor> {
        let kind = CryptoKind::from_js(kind).map_err(VeilidAPIError::generic)?;
        let schema = schema.try_into().map_err(VeilidAPIError::generic)?;
        let owner = match owner {
            Some(owner) => try_from_js_option::<KeyPair>(owner).map_err(VeilidAPIError::generic)?,
            None => None,
        };

        let routing_context = self.getRoutingContext()?;

        let dht_record_descriptor = routing_context
            .create_dht_record(kind, schema, owner)
            .await?;

        Ok(dht_record_descriptor.into())
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
    /// @returns the DHT record descriptor for the opened record if successful.
    /// @param {string} key - key of the DHT record.
    /// @param {string} defaultWriter - the writer keypair to use for set value operations by default
    #[wasm_bindgen(skip_jsdoc)]
    pub async fn openDHTRecord(
        &self,
        recordKey: &RecordKey,
        defaultWriter: Option<TypeStubKeyPair>,
    ) -> VeilidAPIResult<ts::DHTRecordDescriptor> {
        let default_writer = match defaultWriter {
            Some(default_writer) => {
                try_from_js_option::<KeyPair>(default_writer).map_err(VeilidAPIError::generic)?
            }
            None => None,
        };

        let routing_context = self.getRoutingContext()?;
        routing_context
            .open_dht_record(recordKey.clone(), default_writer)
            .await
            .map(|x| x.into())
    }

    /// Closes a DHT record at a specific key that was opened with create_dht_record or open_dht_record.
    ///
    /// Closing a record allows you to re-open it with a different routing context
    pub async fn closeDHTRecord(&self, recordKey: &RecordKey) -> VeilidAPIResult<()> {
        let routing_context = self.getRoutingContext()?;
        routing_context.close_dht_record(recordKey.clone()).await
    }

    /// Deletes a DHT record at a specific key.
    ///
    /// If the record is opened, it must be closed before it is deleted.
    /// Deleting a record does not delete it from the network, but will remove the storage of the record
    /// locally, and will prevent its value from being refreshed on the network by this node.
    pub async fn deleteDHTRecord(&self, recordKey: &RecordKey) -> VeilidAPIResult<()> {
        let routing_context = self.getRoutingContext()?;
        routing_context.delete_dht_record(recordKey.clone()).await
    }

    /// Gets the latest value of a subkey.
    ///
    /// May pull the latest value from the network, but by settings 'force_refresh' you can force a network data refresh.
    ///
    /// Returns `undefined` if the value subkey has not yet been set.
    /// Returns a Uint8Array of `data` if the value subkey has valid data.
    pub async fn getDHTValue(
        &self,
        recordKey: &RecordKey,
        subkey: u32,
        forceRefresh: bool,
    ) -> VeilidAPIResult<Option<ts::ValueData>> {
        let routing_context = self.getRoutingContext()?;
        routing_context
            .get_dht_value(recordKey.clone(), subkey, forceRefresh)
            .await
            .map(|x| x.map(|y| y.into()))
    }

    /// Pushes a changed subkey value to the network.
    /// The DHT record must first by opened via open_dht_record or create_dht_record.
    ///
    /// The writer, if specified, will override the 'default_writer' specified when the record is opened.
    ///
    /// Returns `undefined` if the value was successfully put.
    /// Returns a Uint8Array of `data` if the value put was older than the one available on the network.
    pub async fn setDHTValue(
        &self,
        recordKey: &RecordKey,
        subkey: u32,
        data: Box<[u8]>,
        options: Option<ts::SetDHTValueOptions>,
    ) -> VeilidAPIResult<Option<ts::ValueData>> {
        let data = data.into_vec();

        let routing_context = self.getRoutingContext()?;
        routing_context
            .set_dht_value(
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
    pub async fn watchDhtValues(
        &self,
        recordKey: &RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        expiration: Option<Timestamp>,
        count: Option<u32>,
    ) -> VeilidAPIResult<bool> {
        let routing_context = self.getRoutingContext()?;
        let res = routing_context
            .watch_dht_values(recordKey.clone(), subkeys, expiration, count)
            .await?;
        VeilidAPIResult::Ok(res)
    }

    /// Cancels a watch early.
    ///
    /// This is a convenience function that cancels watching all subkeys in a range. The subkeys specified here
    /// are subtracted from the currently-watched subkey range.
    /// If no range is specified, this is equivalent to cancelling the entire range of subkeys.
    /// Only the subkey range is changed, the expiration and count remain the same.
    /// If no subkeys remain, the watch is entirely cancelled and will receive no more updates.
    ///
    /// Returns Ok(true) if a watch is active for this record.
    /// Returns Ok(false) if the entire watch has been cancelled.
    pub async fn cancelDHTWatch(
        &self,
        recordKey: &RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
    ) -> VeilidAPIResult<bool> {
        let routing_context = self.getRoutingContext()?;
        routing_context
            .cancel_dht_watch(recordKey.clone(), subkeys)
            .await
    }

    /// Inspects a DHT record for subkey state.
    /// This is useful for checking if you should push new subkeys to the network, or retrieve the current state of a record from the network
    /// to see what needs updating locally.
    ///
    /// * `key` is the record key to watch. it must first be opened for reading or writing.
    /// * `subkeys` is the the range of subkeys to inspect. The range must not exceed 512 discrete non-overlapping or adjacent subranges.
    ///    If no range is specified, this is equivalent to inspecting the entire range of subkeys. In total, the list of subkeys returned will be truncated at 512 elements.
    /// * `scope` is what kind of range the inspection has:
    ///
    ///   - DHTReportScope::Local
    ///     Results will be only for a locally stored record.
    ///     Useful for seeing what subkeys you have locally and which ones have not been retrieved
    ///
    ///   - DHTReportScope::SyncGet
    ///     Return the local sequence numbers and the network sequence numbers with GetValue fanout parameters
    ///     Provides an independent view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if the local copy did not exist locally.
    ///     Useful for determining if the current local copy should be updated from the network.
    ///
    ///   - DHTReportScope::SyncSet
    ///     Return the local sequence numbers and the network sequence numbers with SetValue fanout parameters
    ///     Provides an independent view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if the local copy did not exist locally.
    ///     Useful for determining if the unchanged local copy should be pushed to the network.
    ///
    ///   - DHTReportScope::UpdateGet
    ///     Return the local sequence numbers and the network sequence numbers with GetValue fanout parameters
    ///     Provides an view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if a GetValue operation were being performed, including accepting newer values from the network.
    ///     Useful for determining which subkeys would change with a GetValue operation
    ///
    ///   - DHTReportScope::UpdateSet
    ///     Return the local sequence numbers and the network sequence numbers with SetValue fanout parameters
    ///     Provides an view of both the local sequence numbers and the network sequence numbers for nodes that
    ///     would be reached as if a SetValue operation were being performed, including accepting newer values from the network.
    ///     This simulates a SetValue with the initial sequence number incremented by 1, like a real SetValue would when updating.
    ///     Useful for determine which subkeys would change with an SetValue operation
    ///
    /// Returns a DHTRecordReport with the subkey ranges that were returned that overlapped the schema, and sequence numbers for each of the subkeys in the range.
    pub async fn inspectDHTRecord(
        &self,
        recordKey: &RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        scope: Option<DHTReportScope>,
    ) -> VeilidAPIResult<DHTRecordReport> {
        let scope = scope.unwrap_or_default();

        let routing_context = self.getRoutingContext()?;
        routing_context
            .inspect_dht_record(recordKey.clone(), subkeys, scope)
            .await
    }
}
