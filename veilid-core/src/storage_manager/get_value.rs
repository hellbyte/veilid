use super::*;

impl_veilid_log_facility!("stor");

/// The context of the outbound_get_value operation
struct OutboundGetValueContext {
    /// The latest value of the subkey, may be the value passed in
    pub value: Option<Arc<SignedValueData>>,
    /// How to handle the descriptor
    pub descriptor_mode: GetDescriptorMode,
    /// The parsed schema from the descriptor if we have one
    pub schema: Option<DHTSchema>,
    /// If we should send a partial update with the current context
    pub send_partial_update: bool,
}

/// The result of the outbound_get_value operation
#[derive(Debug)]
pub(super) struct OutboundGetValueResult {
    /// Fanout result
    pub fanout_result: FanoutResult,
    /// The subkey that was retrieved
    pub get_result: GetResult,
}

/// The result of the inbound_get_value operation
#[derive(Clone, Debug)]
pub(crate) enum InboundGetValueResult {
    /// Value got successfully, or there was no value
    Success(GetResult),
}

enum GetValueLockShortcutResult {
    Shortcut(Option<ValueData>),
    Locked(StorageManagerSubkeyLockGuard),
}

impl StorageManager {
    /// Get the value of a subkey from an opened local record
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub async fn get_value(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
        force_refresh: bool,
    ) -> VeilidAPIResult<Option<ValueData>> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        // Try the GetValue shortcut first to see if we can skip the lock
        let subkey_lock = match self
            .get_value_lock_shortcut(record_key.clone(), subkey, force_refresh)
            .await?
        {
            GetValueLockShortcutResult::Shortcut(opt_value_data) => return Ok(opt_value_data),
            GetValueLockShortcutResult::Locked(subkey_lock) => subkey_lock,
        };

        // If the shortcut didn't work, then proceed with the full GetValue
        let opaque_record_key = record_key.opaque();

        let safety_selection = {
            let inner = self.inner.lock();
            let Some(opened_record) = inner.opened_records.get(&opaque_record_key) else {
                apibail_generic!("record not open");
            };
            opened_record.safety_selection()
        };

        // See if the requested subkey is our local record store
        let last_get_result = self
            .handle_get_single_local_value(&opaque_record_key, subkey, true)
            .await?;

        // Return the existing value if we have one if we are not forcing a refresh
        if !force_refresh {
            if let Some(last_get_result_value) = last_get_result.opt_value {
                return Ok(Some(self.maybe_decrypt_value_data(
                    &record_key,
                    last_get_result_value.value_data(),
                )?));
            }
        }
        // If we're not online bail early
        if !self.dht_is_online() {
            apibail_try_again!("offline, try again later");
        };

        // May have last descriptor / value
        // Use the safety selection we opened the record with
        let last_seq = last_get_result
            .opt_value
            .as_ref()
            .map(|v| v.value_data().seq())
            .unwrap_or_default();
        let res_rx = self
            .outbound_get_value(
                opaque_record_key.clone(),
                subkey,
                safety_selection,
                last_get_result,
            )
            .await?;

        // Wait for the first result
        let Ok(result) = res_rx.recv_async().await else {
            apibail_internal!("failed to receive results");
        };
        let result = result?;
        let partial = result.fanout_result.kind.is_incomplete();

        // Process the returned result
        let out_encrypted = self
            .process_outbound_get_value_result_locked(&subkey_lock, last_seq, result)
            .await?;
        let out = if let Some(vd) = out_encrypted {
            Some(self.maybe_decrypt_value_data(&record_key, &vd)?)
        } else {
            None
        };

        if let Some(out) = &out {
            // If there's more to process, do it in the background
            if partial {
                self.process_deferred_outbound_get_value_result(
                    res_rx,
                    record_key.clone(),
                    subkey,
                    out.seq(),
                );
            }
        }

        Ok(out)
    }

    /// Handle a received 'Get Value' query
    #[instrument(level = "trace", target = "dht", skip_all)]
    pub async fn inbound_get_value(
        &self,
        opaque_record_key: &OpaqueRecordKey,
        subkey: ValueSubkey,
        want_descriptor: bool,
    ) -> VeilidAPIResult<NetworkResult<InboundGetValueResult>> {
        let remote_record_store = self.get_remote_record_store()?;

        // See if it's in the remote record store
        let last_get_result = remote_record_store
            .get_subkey(opaque_record_key, subkey, want_descriptor)
            .await?
            .unwrap_or_default();

        Ok(NetworkResult::value(InboundGetValueResult::Success(
            last_get_result,
        )))
    }
    ////////////////////////////////////////////////////////////////////////

    // GetValue lock shortcut: Allows gets to happen during other operations
    // because the RecordStore also has its own locks for consistency and this operation
    // does not require multiple awaits within the storage manager
    async fn get_value_lock_shortcut(
        &self,
        record_key: RecordKey,
        subkey: ValueSubkey,
        force_refresh: bool,
    ) -> VeilidAPIResult<GetValueLockShortcutResult> {
        let opaque_record_key = record_key.opaque();

        let subkey_lock = if !force_refresh {
            match self.record_lock_table.try_lock_subkey(
                opaque_record_key.clone(),
                subkey,
                StorageManagerSubkeyLockPurpose::Get,
            ) {
                Some(subkey_lock) => {
                    // We got the full lock right away, so just proceed without the shortcut
                    subkey_lock
                }
                None => {
                    // Could not lock the subkey, so let's try a local get first immediately

                    // Ensure the record is open
                    let is_open = {
                        let inner = self.inner.lock();
                        inner.opened_records.contains_key(&opaque_record_key)
                    };
                    if is_open {
                        // If the record is open, ask the record store to get the value
                        let last_get_result = self
                            .handle_get_single_local_value(&opaque_record_key, subkey, true)
                            .await?;
                        if let Some(last_get_result_value) = last_get_result.opt_value {
                            return Ok(GetValueLockShortcutResult::Shortcut(Some(
                                self.maybe_decrypt_value_data(
                                    &record_key,
                                    last_get_result_value.value_data(),
                                )?,
                            )));
                        }
                    }

                    // If we couldn't get a local value then wait to obtain the lock and proceed with
                    // the non-shortcut route
                    self.record_lock_table
                        .lock_subkey(
                            opaque_record_key.clone(),
                            subkey,
                            StorageManagerSubkeyLockPurpose::Get,
                        )
                        .await
                }
            }
        } else {
            // If we couldn't get a local value then wait to obtain the lock and proceed with
            // the non-shortcut route
            self.record_lock_table
                .lock_subkey(
                    opaque_record_key.clone(),
                    subkey,
                    StorageManagerSubkeyLockPurpose::Get,
                )
                .await
        };

        Ok(GetValueLockShortcutResult::Locked(subkey_lock))
    }

    /// Perform a 'get value' query on the network
    /// Performs the work without a transaction
    #[instrument(level = "trace", target = "dht", skip_all, err)]
    pub(super) async fn outbound_get_value(
        &self,
        opaque_record_key: OpaqueRecordKey,
        subkey: ValueSubkey,
        safety_selection: SafetySelection,
        last_get_result: GetResult,
    ) -> VeilidAPIResult<flume::Receiver<VeilidAPIResult<OutboundGetValueResult>>> {
        let routing_domain = RoutingDomain::PublicInternet;
        let config = self.config();

        // Get the DHT parameters for 'GetValue'
        let key_count = config.network.dht.max_find_node_count as usize;
        let consensus_count = config.network.dht.get_value_count as usize;
        let fanout = config.network.dht.get_value_fanout as usize;
        let timeout_us = TimestampDuration::from(ms_to_us(config.network.dht.get_value_timeout_ms));

        // Get the nodes we know are caching this value to seed the fanout
        let init_fanout_queue = self
            .get_value_nodes(&opaque_record_key)?
            .unwrap_or_default()
            .into_iter()
            .filter(|x| {
                x.node_info(routing_domain)
                    .map(|ni| ni.has_capability(VEILID_CAPABILITY_DHT))
                    .unwrap_or_default()
            })
            .collect();

        // Parse the schema
        let schema = if let Some(d) = &last_get_result.opt_descriptor {
            Some(d.schema()?)
        } else {
            None
        };

        // Make the descriptor mode
        let descriptor_mode = GetDescriptorMode::new(last_get_result.opt_descriptor.clone());

        // Make the return channel
        let (out_tx, out_rx) = flume::unbounded::<VeilidAPIResult<OutboundGetValueResult>>();

        // Make operation context
        let context = Arc::new(Mutex::new(OutboundGetValueContext {
            value: last_get_result.opt_value,
            descriptor_mode,
            schema,
            send_partial_update: true,
        }));

        // Routine to call to generate fanout
        let call_routine = {
            let context = context.clone();
            let registry = self.registry();
            let opaque_record_key = opaque_record_key.clone();
            let safety_selection = safety_selection.clone();
            Arc::new(
                move |next_node: NodeRef| -> PinBoxFutureStatic<FanoutCallResult> {
                    let context = context.clone();
                    let registry = registry.clone();
                    let opaque_record_key = opaque_record_key.clone();
                    let safety_selection = safety_selection.clone();
                    Box::pin(async move {
                        let rpc_processor = registry.rpc_processor();
                        let descriptor_mode = context.lock().descriptor_mode.clone();
                        let gva = match
                            rpc_processor
                                .rpc_call_get_value(
                                    Destination::direct(next_node.routing_domain_filtered(routing_domain))
                                        .with_safety(safety_selection),
                                    opaque_record_key.clone(),
                                    subkey,
                                    descriptor_mode,
                                )
                                .await? {
                                    NetworkResult::Timeout => {
                                        return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Timeout});
                                    }
                                    NetworkResult::ServiceUnavailable(_) |
                                    NetworkResult::NoConnection(_)  |
                                    NetworkResult::AlreadyExists(_) |
                                    NetworkResult::InvalidMessage(_) => {
                                        return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Invalid});
                                    }
                                    NetworkResult::Value(v) => v
                                };

                        // Check if we got an accepted result
                        if !gva.answer.accepted {
                            // Return peers if we have some
                            veilid_log!(registry debug target:"network_result", "GetValue missed, fanout call returned peers {}", gva.answer.peers.len());
                            return Ok(FanoutCallOutput{peer_info_list: gva.answer.peers, disposition: FanoutCallDisposition::Rejected});
                        }

                        let mut ctx = context.lock();
                        // Keep the descriptor if we got one. If we had a last_descriptor it will
                        // already be validated by rpc_call_get_value
                        if let Some(descriptor) = gva.answer.descriptor {
                            if ctx.descriptor_mode.is_want() && ctx.schema.is_none() {
                                let schema = match descriptor.schema() {
                                    Ok(v) => v,
                                    Err(e) => {
                                        veilid_log!(registry debug target:"network_result", "GetValue returned an invalid descriptor: {}", e);
                                        return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Invalid});
                                    }
                                };
                                ctx.schema = Some(schema);
                                ctx.descriptor_mode = GetDescriptorMode::have(Arc::new(descriptor));
                            }
                        }

                        // Keep the value if we got one and it is newer and it passes schema validation
                        let Some(value) = gva.answer.value else {
                            // Return peers if we have some
                            veilid_log!(registry debug target:"network_result", "GetValue returned no value, fanout call returned peers {}", gva.answer.peers.len());
                            return Ok(FanoutCallOutput{peer_info_list: gva.answer.peers, disposition: FanoutCallDisposition::Stale});
                        };

                        veilid_log!(registry debug "GetValue got value back: {}", value);

                        // Ensure we have a schema and descriptor
                        let (Some(descriptor), Some(schema)) = (ctx.descriptor_mode.opt_ref_descriptor(), &ctx.schema)
                        else {
                            // Got a value but no descriptor for it
                            // Move to the next node
                            return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Invalid});
                        };

                        // Validate with schema
                        if registry.storage_manager().check_subkey_value_data(schema,
                            descriptor.ref_owner(),
                            subkey,
                            value.value_data(),
                        ).is_err() {
                            // Validation failed, ignore this value
                            // Move to the next node
                            return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Invalid});
                        }

                        // If we have a prior value, see if this is a newer sequence number
                        let disposition = if let Some(prior_value) = &ctx.value {
                            let prior_seq = prior_value.value_data().seq();
                            let new_seq = value.value_data().seq();

                            if new_seq == prior_seq {
                                // If sequence number is the same, the data should be the same
                                if prior_value.value_data() != value.value_data() {
                                    // Value data mismatch means skip this node
                                    // This is okay because even the conflicting value is signed,
                                    // so the application just needs to push a newer value
                                    FanoutCallDisposition::Stale
                                } else {
                                    // Increase the consensus count for the existing value
                                    FanoutCallDisposition::Accepted
                                }
                            } else if new_seq > prior_seq {
                                // If the sequence number is greater, start over with the new value
                                ctx.value = Some(Arc::new(value));
                                // Send an update since the value changed
                                ctx.send_partial_update = true;

                                // Restart the consensus since we have a new value, but
                                // don't retry nodes we've already seen because they will return
                                // the same answer
                                FanoutCallDisposition::AcceptedNewer
                            } else {
                                // If the sequence number is older, ignore it
                                FanoutCallDisposition::Stale
                            }
                        } else {
                            // If we have no prior value, keep it
                            ctx.value = Some(Arc::new(value));
                            // No value was returned
                            FanoutCallDisposition::Accepted
                        };
                        // Return peers if we have some
                        veilid_log!(registry debug target:"network_result", "GetValue fanout call returned peers {}", gva.answer.peers.len());

                        Ok(FanoutCallOutput{peer_info_list: gva.answer.peers, disposition})
                    }.instrument(tracing::trace_span!("outbound_get_value fanout routine"))) as PinBoxFuture<FanoutCallResult>
                },
            )
        };

        // Routine to call to check if we're done at each step
        let check_done = {
            let context = context.clone();
            let out_tx = out_tx.clone();
            let registry = self.registry();
            Arc::new(
                move |fanout_result: &FanoutResult| -> FanoutDoneDisposition {
                    let mut ctx = context.lock();

                    match fanout_result.kind {
                        FanoutResultKind::Incomplete => {
                            // Send partial update if desired, if we've gotten at least one consensus node
                            if ctx.send_partial_update && !fanout_result.consensus_nodes.is_empty()
                            {
                                ctx.send_partial_update = false;

                                // Return partial result
                                let out = OutboundGetValueResult {
                                    fanout_result: fanout_result.clone(),
                                    get_result: GetResult {
                                        opt_value: ctx.value.clone(),
                                        opt_descriptor: ctx.descriptor_mode.opt_arc_descriptor(),
                                    },
                                };
                                veilid_log!(registry debug "Sending partial GetValue result: {:?}", out);
                                if let Err(e) = out_tx.send(Ok(out)) {
                                    veilid_log!(registry debug "Sending partial GetValue result failed: {}", e);
                                }
                            }
                            // Keep going
                            FanoutDoneDisposition::NotDone
                        }
                        FanoutResultKind::Timeout | FanoutResultKind::Exhausted => {
                            // Signal we're done
                            FanoutDoneDisposition::DoneEarly
                        }
                        FanoutResultKind::Consensus => {
                            assert!(
                                ctx.value.is_some() && ctx.descriptor_mode.is_have(),
                                "should have gotten a value if we got consensus"
                            );
                            // Signal we're done
                            FanoutDoneDisposition::DoneEarly
                        }
                    }
                },
            )
        };

        // Call the fanout in a spawned task
        let registry = self.registry();
        let fanout_hash_coordinate = opaque_record_key.to_hash_coordinate();
        spawn(
            "outbound_get_value fanout",
            Box::pin(
                async move {
                    let routing_table = registry.routing_table();
                    let fanout_call = FanoutCall::new(
                        format!("outbound_get_value({})", Timestamp::now_increasing()),
                        &routing_table,
                        fanout_hash_coordinate,
                        key_count,
                        fanout,
                        consensus_count,
                        timeout_us,
                        capability_fanout_peer_info_filter(vec![VEILID_CAPABILITY_DHT]),
                        call_routine,
                        check_done,
                    );

                    let fanout_result = match fanout_call.run(init_fanout_queue, FanoutQueueMode::ThrottleAtConsensus).await {
                        Ok(v) => v,
                        Err(e) => {
                            // If we finished with an error, return that
                            veilid_log!(registry debug "GetValue fanout error: {}", e);
                            if let Err(e) = out_tx.send(Err(e.into())) {
                                veilid_log!(registry debug "Sending GetValue fanout error failed: {}", e);
                            }
                            return;
                        }
                    };

                    veilid_log!(registry debug "GetValue Fanout: {:#}", fanout_result);

                    let out = {
                        let ctx = context.lock();
                        OutboundGetValueResult {
                            fanout_result,
                            get_result: GetResult {
                                opt_value: ctx.value.clone(),
                                opt_descriptor: ctx.descriptor_mode.opt_arc_descriptor(),
                            },
                        }
                    };

                    if let Err(e) = out_tx.send(Ok(out)) {
                        veilid_log!(registry debug "Sending GetValue result failed: {}", e);
                    }
                }
                .instrument(tracing::trace_span!("outbound_get_value result")),
            ),
        )
        .detach();

        Ok(out_rx)
    }

    #[instrument(level = "trace", target = "dht", skip_all)]
    pub(super) fn process_deferred_outbound_get_value_result(
        &self,
        res_rx: flume::Receiver<Result<get_value::OutboundGetValueResult, VeilidAPIError>>,
        record_key: RecordKey,
        subkey: ValueSubkey,
        last_seq: ValueSeqNum,
    ) {
        let registry = self.registry();
        self.process_deferred_results(
            res_rx,
            Box::new(
                move |result: VeilidAPIResult<get_value::OutboundGetValueResult>| -> PinBoxFutureStatic<DeferredStreamResult> {
                    let registry=registry.clone();
                    let record_key = record_key.clone();
                    Box::pin(async move {
                        let this = registry.storage_manager();
                        let result = match result {
                            Ok(v) => v,
                            Err(e) => {
                                veilid_log!(this debug "Deferred fanout error: {}", e);
                                return DeferredStreamResult::Done;
                            }
                        };
                        let is_incomplete = result.fanout_result.kind.is_incomplete();

                        let subkey_lock = this
                            .record_lock_table
                            .lock_subkey(record_key.opaque(), subkey, StorageManagerSubkeyLockPurpose::Get)
                            .await;

                        let value_data = match this.process_outbound_get_value_result_locked(&subkey_lock, last_seq, result).await {
                            Ok(Some(v)) => v,
                            Ok(None) => {
                                return if is_incomplete {
                                    DeferredStreamResult::Continue
                                } else {
                                    DeferredStreamResult::Done
                                };
                            }
                            Err(VeilidAPIError::KeyNotFound { key }) => {
                                veilid_log!(registry debug "Record no longer exists during deferred outbound get value: {}", key);
                                return DeferredStreamResult::Done;
                            }
                            Err(e) => {
                                veilid_log!(this debug "Deferred fanout error: {}", e);
                                return DeferredStreamResult::Done;
                            }
                        };
                        if is_incomplete {
                            // If more partial results show up, don't send an update until we're done
                            return DeferredStreamResult::Continue;
                        }
                        // If we processed the final result, possibly send an update
                        // if the sequence number changed since our first partial update
                        // Send with a max count as this is not attached to any watch
                        if last_seq != value_data.seq() {
                            let value_data = match this.maybe_decrypt_value_data(&record_key, &value_data) {
                                Ok(v) => v,
                                Err(e) => {
                                    veilid_log!(this debug "Deferred fanout error: {}", e);
                                    return DeferredStreamResult::Done;
                                }
                            };

                            this.update_callback_value_change(
                                record_key.clone(),
                                ValueSubkeyRangeSet::single(subkey),
                                u32::MAX,
                                Some(value_data),
                            );
                        }

                        // Return done
                        DeferredStreamResult::Done
                    }.instrument(tracing::trace_span!("outbound_get_value deferred results")))
                },
            ),
        );
    }

    #[instrument(level = "trace", target = "dht", skip_all)]
    pub(super) async fn process_outbound_get_value_result_locked(
        &self,
        subkey_lock: &StorageManagerSubkeyLockGuard,
        last_seq: ValueSeqNum,
        result: get_value::OutboundGetValueResult,
    ) -> VeilidAPIResult<Option<EncryptedValueData>> {
        // See if we got a value back
        let Some(get_result_value) = result.get_result.opt_value else {
            // If we got nothing back then we also had nothing beforehand, return nothing
            return Ok(None);
        };

        let opaque_record_key = subkey_lock.record();
        let subkey = subkey_lock.subkey();

        // Keep the list of nodes that returned a value for later reference
        let existed = self.process_fanout_results(
            opaque_record_key.clone(),
            core::iter::once((ValueSubkeyRangeSet::single(subkey), result.fanout_result)),
            false,
            self.config().network.dht.consensus_width as usize,
        )?;

        // Check if the record still exists before setting it locally
        if !existed {
            apibail_key_not_found!(opaque_record_key);
        }

        // If we got a new value back then write it to the opened record
        if get_result_value.value_data().seq() != last_seq {
            self.handle_set_single_local_value_with_subkey_lock(
                subkey_lock,
                get_result_value.clone(),
            )
            .await?;
        }
        Ok(Some(get_result_value.value_data().clone()))
    }
}
