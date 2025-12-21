use futures_util::StreamExt as _;

use super::*;

impl_veilid_log_facility!("stor");

/// The context of the outbound_watch_value operation
#[derive(Debug, Default)]
struct OutboundWatchValueContext {
    /// A successful watch
    pub watch_value_result: OutboundWatchValueResult,
}

/// The record of a node accepting a watch
#[derive(Debug, Clone)]
pub(super) struct AcceptedWatch {
    pub watch_id: u64,
    /// The node that accepted the watch
    pub node_ref: NodeRef,
    /// The expiration of a successful watch
    pub expiration: Timestamp,
    /// Which private route is responsible for receiving ValueChanged notifications
    pub opt_value_changed_route: Option<PublicKey>,
}

/// The result of the outbound_watch_value operation
#[derive(Debug, Clone, Default)]
pub(super) struct OutboundWatchValueResult {
    /// Which nodes accepted the watch
    pub accepted: Vec<AcceptedWatch>,
    /// Which nodes rejected or cancelled the watch
    pub rejected: Vec<NodeRef>,
    /// Which nodes ignored the watch
    pub ignored: Vec<NodeRef>,
}

/// Watch parameters used to configure a watch
#[derive(Debug, Clone)]
pub(crate) struct InboundWatchParameters {
    /// The range of subkeys being watched, empty meaning full
    pub subkeys: ValueSubkeyRangeSet,
    /// When this watch will expire
    pub expiration: Timestamp,
    /// How many updates are left before forced expiration
    pub count: u32,
    /// The watching schema member key, or an anonymous key
    pub watcher_member_id: MemberId,
    /// The place where updates are sent
    pub target: Target,
}

/// Watch result to return with answer
/// Default result is cancelled/expired/inactive/rejected
#[derive(Debug, Clone)]
pub(crate) enum InboundWatchValueResult {
    /// A new watch was created
    Created {
        /// The new id of the watch
        id: InboundWatchId,
        /// The expiration timestamp of the watch. This should never be zero.
        expiration: Timestamp,
    },
    /// An existing watch was modified
    Changed {
        /// The new expiration timestamp of the modified watch. This should never be zero.
        expiration: Timestamp,
    },
    /// An existing watch was cancelled
    Cancelled,
    /// The request was rejected due to invalid parameters or a missing watch
    Rejected,
}

impl OutboundWatchValueResult {
    pub fn merge(&mut self, other: OutboundWatchValueResult) {
        self.accepted.extend(other.accepted);
        self.rejected.extend(other.rejected);
        self.ignored.extend(other.ignored);
    }
}

impl StorageManager {
    /// Create, update or cancel an outbound watch to a DHT value
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub async fn watch_values(
        &self,
        record_key: RecordKey,
        subkeys: ValueSubkeyRangeSet,
        expiration: Timestamp,
        count: u32,
    ) -> VeilidAPIResult<bool> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        // Obtain the watch change lock
        // (may need to wait for background operations to complete on the watch)
        let _records_lock = self
            .record_lock_table
            .lock_record(record_key.opaque(), StorageManagerRecordLockPurpose::Watch)
            .await;

        let mut inner = self.inner.lock();
        self.watch_values_inner(&mut inner, record_key, subkeys, expiration, count)
    }

    #[instrument(level = "trace", target = "stor", skip_all)]
    pub async fn cancel_watch_values(
        &self,
        record_key: RecordKey,
        subkeys: ValueSubkeyRangeSet,
    ) -> VeilidAPIResult<bool> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };
        let opaque_record_key = record_key.opaque();

        // Obtain the watch change lock
        // (may need to wait for background operations to complete on the watch)
        let _records_lock = self
            .record_lock_table
            .lock_record(
                opaque_record_key.clone(),
                StorageManagerRecordLockPurpose::Watch,
            )
            .await;

        // Calculate change to existing watch
        let mut inner = self.inner.lock();

        let Some(_opened_record) = inner.opened_records.get(&opaque_record_key) else {
            apibail_generic!("record not open");
        };

        // See what watch we have currently if any
        let Some(outbound_watch) = inner
            .outbound_watch_manager
            .outbound_watches
            .get(&opaque_record_key)
        else {
            // If we didn't have an active watch, then we can just return false because there's nothing to do here
            return Ok(false);
        };

        // Ensure we have a 'desired' watch state
        let Some(desired) = outbound_watch.desired() else {
            // If we didn't have a desired watch, then we're already cancelling
            let still_active = outbound_watch.state().is_some();
            return Ok(still_active);
        };

        // Rewrite subkey range if empty to full
        let subkeys = if subkeys.is_empty() {
            ValueSubkeyRangeSet::full()
        } else {
            subkeys
        };

        // Reduce the subkey range
        let new_subkeys = desired.subkeys.difference(&subkeys);

        // If no change is happening return false
        if new_subkeys == desired.subkeys {
            return Ok(false);
        }

        // If we have no subkeys left, then set the count to zero to indicate a full cancellation
        let count = if new_subkeys.is_empty() {
            0
        } else if let Some(state) = outbound_watch.state() {
            state.remaining_count()
        } else {
            desired.count
        };

        // Update the watch. This just calls through to the above watch_values_inner() function
        // This will update the active_watch so we don't need to do that in this routine.
        self.watch_values_inner(
            &mut inner,
            record_key,
            new_subkeys,
            desired.expiration,
            count,
        )
    }

    /// Get the set of nodes in our active watches
    pub fn get_outbound_watch_nodes(&self) -> Vec<Destination> {
        let inner = self.inner.lock();

        let mut out = vec![];
        let mut node_set: HashSet<HashAtom<BucketEntry>> = HashSet::new();
        for v in inner.outbound_watch_manager.outbound_watches.values() {
            if let Some(current) = v.state() {
                let node_refs =
                    current.watch_node_refs(&inner.outbound_watch_manager.per_node_states);
                for node_ref in &node_refs {
                    if node_set.contains(&node_ref.entry().hash_atom()) {
                        continue;
                    }

                    node_set.insert(node_ref.entry().hash_atom());
                    out.push(
                        Destination::direct(
                            node_ref.routing_domain_filtered(RoutingDomain::PublicInternet),
                        )
                        .with_safety(current.params().safety_selection.clone()),
                    )
                }
            }
        }

        out
    }

    ////////////////////////////////////////////////////////////////////////

    #[instrument(level = "trace", target = "stor", skip_all)]
    fn watch_values_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: RecordKey,
        subkeys: ValueSubkeyRangeSet,
        expiration: Timestamp,
        count: u32,
    ) -> VeilidAPIResult<bool> {
        let opaque_record_key = record_key.opaque();

        // Get the safety selection and the writer we opened this record
        let (safety_selection, opt_watcher) = {
            let Some(opened_record) = inner.opened_records.get(&opaque_record_key) else {
                // Record must be opened already to change watch
                apibail_generic!("record not open");
            };
            (
                opened_record.safety_selection(),
                opened_record.writer().cloned(),
            )
        };

        // Rewrite subkey range if empty to full
        let subkeys = if subkeys.is_empty() {
            ValueSubkeyRangeSet::full()
        } else {
            subkeys
        };

        // Get the schema so we can truncate the watch to the number of subkeys
        let schema = if let Some(lrs) = inner.local_record_store.as_ref() {
            let Some(schema) = lrs.peek_record(&opaque_record_key, |r| r.schema()) else {
                apibail_generic!("no local record found");
            };
            schema
        } else {
            apibail_not_initialized!();
        };
        let subkeys = schema.truncate_subkeys(&subkeys, None);

        // Calculate desired watch parameters
        let desired_params = if count == 0 {
            // Cancel
            None
        } else {
            // Get the minimum expiration timestamp we will accept
            let rpc_timeout =
                TimestampDuration::new_ms(self.config().network.rpc.timeout_ms.into());
            let min_expiration = Timestamp::now().later(rpc_timeout);
            let expiration = if expiration.as_u64() == 0 {
                expiration
            } else if expiration < min_expiration {
                apibail_invalid_argument!("expiration is too soon", "expiration", expiration);
            } else {
                expiration
            };

            // Create or modify
            Some(OutboundWatchParameters {
                expiration,
                count,
                subkeys,
                opt_watcher,
                safety_selection,
            })
        };

        // Modify the 'desired' state of the watch or add one if it does not exist
        let active = desired_params.is_some();
        inner
            .outbound_watch_manager
            .set_desired_watch(record_key, desired_params);

        Ok(active)
    }

    /// Perform a 'watch value cancel' on the network without fanout
    #[instrument(target = "watch", level = "debug", skip_all, err)]
    pub(super) async fn outbound_watch_value_cancel(
        &self,
        opaque_record_key: OpaqueRecordKey,
        opt_watcher: Option<KeyPair>,
        safety_selection: SafetySelection,
        watch_node: NodeRef,
        watch_id: u64,
    ) -> VeilidAPIResult<bool> {
        let routing_domain = RoutingDomain::PublicInternet;

        // Get the appropriate watcher key, if anonymous use a static anonymous watch key
        // which lives for the duration of the app's runtime
        let watcher = opt_watcher.unwrap_or_else(|| {
            self.anonymous_signing_keys
                .get(opaque_record_key.kind())
                .unwrap()
        });

        let wva = VeilidAPIError::from_network_result(
            self.rpc_processor()
                .rpc_call_watch_value(
                    Destination::direct(watch_node.routing_domain_filtered(routing_domain))
                        .with_safety(safety_selection),
                    opaque_record_key,
                    ValueSubkeyRangeSet::new(),
                    Timestamp::default(),
                    0,
                    watcher,
                    Some(watch_id),
                )
                .await?,
        )?;

        if wva.answer.accepted {
            veilid_log!(self debug "Outbound watch cancelled: id={} ({})", wva.answer.watch_id, watch_node);
            Ok(true)
        } else {
            veilid_log!(self debug "Outbound watch id did not exist: id={} ({})", watch_id, watch_node);
            Ok(false)
        }
    }

    /// Perform a 'watch value change' on the network without fanout
    #[allow(clippy::too_many_arguments)]
    #[instrument(target = "watch", level = "debug", skip_all, err)]
    pub(super) async fn outbound_watch_value_change(
        &self,
        opaque_record_key: OpaqueRecordKey,
        params: OutboundWatchParameters,
        watch_node: NodeRef,
        watch_id: u64,
    ) -> VeilidAPIResult<OutboundWatchValueResult> {
        let routing_domain = RoutingDomain::PublicInternet;

        if params.count == 0 {
            apibail_internal!("cancel should be done with outbound_watch_value_cancel");
        }

        // Get the appropriate watcher key, if anonymous use a static anonymous watch key
        // which lives for the duration of the app's runtime
        let watcher = params.opt_watcher.unwrap_or_else(|| {
            self.anonymous_signing_keys
                .get(opaque_record_key.kind())
                .unwrap()
        });

        let wva = VeilidAPIError::from_network_result(
            pin_future!(self.rpc_processor().rpc_call_watch_value(
                Destination::direct(watch_node.routing_domain_filtered(routing_domain))
                    .with_safety(params.safety_selection),
                opaque_record_key,
                params.subkeys,
                params.expiration,
                params.count,
                watcher,
                Some(watch_id),
            ))
            .await?,
        )?;

        if wva.answer.accepted {
            if wva.answer.expiration.is_zero() {
                veilid_log!(self debug "WatchValue not renewed: id={} ({})", watch_id, watch_node);
            } else if watch_id != wva.answer.watch_id {
                veilid_log!(self debug "WatchValue changed: id={}->{} expiration={} ({})", watch_id, wva.answer.watch_id, wva.answer.expiration, watch_node);
            } else {
                veilid_log!(self debug "WatchValue renewed: id={} expiration={} ({})", watch_id, wva.answer.expiration, watch_node);
            }

            Ok(OutboundWatchValueResult {
                accepted: vec![AcceptedWatch {
                    watch_id: wva.answer.watch_id,
                    node_ref: watch_node,
                    expiration: wva.answer.expiration,
                    opt_value_changed_route: wva.reply_private_route,
                }],
                rejected: vec![],
                ignored: vec![],
            })
        } else {
            veilid_log!(self debug "WatchValue change failed: id={} ({})", wva.answer.watch_id, watch_node);
            Ok(OutboundWatchValueResult {
                accepted: vec![],
                rejected: vec![watch_node],
                ignored: vec![],
            })
        }
    }

    /// Perform a 'watch value' query on the network using fanout
    ///
    #[allow(clippy::too_many_arguments)]
    #[instrument(target = "watch", level = "debug", skip_all, err)]
    pub(super) async fn outbound_watch_value(
        &self,
        record_key: RecordKey,
        params: OutboundWatchParameters,
        per_node_state: HashMap<PerNodeKey, OutboundWatchPerNodeState>,
    ) -> VeilidAPIResult<OutboundWatchValueResult> {
        let opaque_record_key = record_key.opaque();
        let routing_domain = RoutingDomain::PublicInternet;

        // Get the DHT parameters for 'WatchValue', some of which are the same for 'GetValue' operations
        let config = self.config();
        let key_count = config.network.dht.max_find_node_count as usize;
        let consensus_count = config.network.dht.get_value_count as usize;
        let fanout = config.network.dht.get_value_fanout as usize;
        let timeout_us = TimestampDuration::from(ms_to_us(config.network.dht.get_value_timeout_ms));

        // Get the appropriate watcher key, if anonymous use a static anonymous watch key
        // which lives for the duration of the app's runtime
        let watcher = params
            .opt_watcher
            .clone()
            .unwrap_or_else(|| self.anonymous_signing_keys.get(record_key.kind()).unwrap());

        // Get the nodes we know are caching this value to seed the fanout
        let init_fanout_queue = self
            .get_value_nodes(&opaque_record_key)?
            .unwrap_or_default()
            .into_iter()
            .filter(|x| {
                x.node_info(routing_domain)
                    .map(|ni| ni.has_all_capabilities(&[VEILID_CAPABILITY_DHT]))
                    .unwrap_or_default()
            })
            .collect();

        // Make operation context
        let context = Arc::new(Mutex::new(OutboundWatchValueContext::default()));

        // Routine to call to generate fanout
        let call_routine = {
            let context = context.clone();
            let registry = self.registry();
            let params = params.clone();
            let record_key = record_key.clone();
            let watcher = watcher.clone();
            Arc::new(
                move |next_node: NodeRef| -> PinBoxFutureStatic<FanoutCallResult> {
                    let context = context.clone();
                    let registry = registry.clone();
                    let record_key = record_key.clone();
                    let watcher = watcher.clone();
                    let params = params.clone();

                    // See if we have an existing watch id for this node
                    let node_id = next_node.node_ids().get(record_key.kind()).unwrap();
                    let pnk = PerNodeKey {
                        record_key: record_key.clone(),
                        node_id,
                    };
                    let watch_id = per_node_state.get(&pnk).map(|pns| pns.watch_id);

                    Box::pin(async move {
                        let rpc_processor = registry.rpc_processor();

                        let wva = match
                            rpc_processor
                                .rpc_call_watch_value(
                                    Destination::direct(next_node.routing_domain_filtered(routing_domain)).with_safety(params.safety_selection),
                                    record_key.opaque(),
                                    params.subkeys,
                                    params.expiration,
                                    params.count,
                                    watcher,
                                    watch_id
                                )
                                .await? {
                                    NetworkResult::Timeout => {
                                        let mut ctx = context.lock();
                                        ctx.watch_value_result.ignored.push(next_node.clone());
                                        return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Timeout});
                                    }
                                    NetworkResult::ServiceUnavailable(_) |
                                    NetworkResult::NoConnection(_)  |
                                    NetworkResult::AlreadyExists(_) |
                                    NetworkResult::InvalidMessage(_) => {
                                        let mut ctx = context.lock();
                                        ctx.watch_value_result.ignored.push(next_node.clone());
                                        return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Invalid});
                                    }
                                    NetworkResult::Value(v) => v
                                };

                        // Keep answer if we got one
                        // (accepted means the node could provide an answer, not that the watch is active)
                        let disposition = if wva.answer.accepted {
                            if wva.answer.expiration.as_u64() > 0 {
                                // If the expiration time is greater than zero this watch is active
                                veilid_log!(registry debug target:"watch", "WatchValue accepted for {}: id={} expiration={} ({})", record_key, wva.answer.watch_id, display_ts(wva.answer.expiration.as_u64()), next_node);

                                // Add to accepted watches
                                let mut ctx = context.lock();
                                ctx.watch_value_result.accepted.push(AcceptedWatch{
                                    watch_id: wva.answer.watch_id,
                                    node_ref: next_node.clone(),
                                    expiration: wva.answer.expiration,
                                    opt_value_changed_route: wva.reply_private_route,
                                });

                                FanoutCallDisposition::Accepted
                            } else {
                                // If the returned expiration time is zero, this watch was cancelled

                                // If the expiration time is greater than zero this watch is active
                                veilid_log!(registry debug target:"watch", "WatchValue rejected for {}: id={} expiration={} ({})", record_key, wva.answer.watch_id, display_ts(wva.answer.expiration.as_u64()), next_node);

                                // Add to rejected watches
                                let mut ctx = context.lock();
                                ctx.watch_value_result.rejected.push(next_node.clone());

                                // Treat as accepted but do not add to consensus
                                FanoutCallDisposition::Stale
                            }
                        } else {
                            // Add to rejected watches
                            let mut ctx = context.lock();
                            ctx.watch_value_result.rejected.push(next_node.clone());

                            // Treat as rejected and do not add to consensus
                            FanoutCallDisposition::Rejected
                        };

                        // Return peers if we have some
                        veilid_log!(registry debug target:"network_result", "WatchValue fanout call returned peers {} ({})", wva.answer.peers.len(), next_node);

                        Ok(FanoutCallOutput{peer_info_list: wva.answer.peers, disposition})
                    }.instrument(tracing::trace_span!("outbound_watch_value call routine"))) as PinBoxFuture<FanoutCallResult>
                },
            )
        };

        // Routine to call to check if we're done at each step
        let check_done = {
            Arc::new(
                move |fanout_result: &FanoutResult| -> FanoutDoneDisposition {
                    match fanout_result.kind {
                        FanoutResultKind::Incomplete => {
                            // Keep going
                            FanoutDoneDisposition::NotDone
                        }
                        FanoutResultKind::Timeout | FanoutResultKind::Exhausted => {
                            // Signal we're done
                            FanoutDoneDisposition::DoneEarly
                        }
                        FanoutResultKind::Consensus => {
                            // Signal we're done
                            FanoutDoneDisposition::DoneEarly
                        }
                    }
                },
            )
        };

        // Call the fanout
        // Use the same fanout parameters as a get
        // and each one might take timeout_us time.
        let routing_table = self.routing_table();
        let fanout_call = FanoutCall::new(
            format!("outbound_watch_value({})", Timestamp::now_increasing()),
            &routing_table,
            record_key.to_hash_coordinate(),
            key_count,
            fanout,
            consensus_count,
            timeout_us,
            capability_fanout_peer_info_filter(vec![VEILID_CAPABILITY_DHT]),
            call_routine,
            check_done,
        );

        let fanout_result = fanout_call
            .run(init_fanout_queue, FanoutQueueMode::ThrottleAtConsensus)
            .await
            .inspect_err(|e| {
                // If we finished with an error, return that
                veilid_log!(self debug target:"watch", "WatchValue fanout error: {}", e);
            })?;

        veilid_log!(self debug target:"dht", "WatchValue Fanout: {:#}", fanout_result);

        // Keep the list of nodes that responded for later reference
        let existed = self.process_fanout_results(
            record_key.opaque(),
            core::iter::once((ValueSubkeyRangeSet::new(), fanout_result)),
            false,
            self.config().network.dht.consensus_width as usize,
        )?;
        if !existed {
            apibail_internal!(
                "Record went missing during watch operation despite lock: {}",
                record_key.opaque(),
            )
        }

        let owvresult = context.lock().watch_value_result.clone();
        Ok(owvresult)
    }

    /// Remove dead watches from the table
    pub(super) fn process_outbound_watch_dead(&self, watch_lock: StorageManagerRecordLockGuard) {
        let opaque_record_key = watch_lock.record();

        let Some(outbound_watch) = self
            .inner
            .lock()
            .outbound_watch_manager
            .outbound_watches
            .remove(&opaque_record_key)
        else {
            veilid_log!(self warn "dead watch should have still been in the table");
            return;
        };

        if outbound_watch.state().is_some() {
            veilid_log!(self warn "dead watch still had current state");
        }
        if outbound_watch.desired().is_some() {
            veilid_log!(self warn "dead watch still had desired params");
        }

        // Send valuechange with dead count and no subkeys to inform the api that this watch is now gone completely
        drop(watch_lock);
        self.update_callback_value_change(
            outbound_watch.record_key(),
            ValueSubkeyRangeSet::new(),
            0,
            None,
        );
    }

    /// Get the list of remaining active watch ids
    /// and call their nodes to cancel the watch
    pub(super) async fn process_outbound_watch_cancel(
        &self,
        watch_lock: StorageManagerRecordLockGuard,
    ) {
        let opaque_record_key = watch_lock.record();

        // If we can't do this operation right now, don't try
        if !self.dht_is_online() {
            return;
        }

        let per_node_states = {
            let inner = &mut *self.inner.lock();

            let Some(outbound_watch) = inner
                .outbound_watch_manager
                .outbound_watches
                .get_mut(&opaque_record_key)
            else {
                veilid_log!(self warn "watch being cancelled should have still been in the table");
                return;
            };
            let Some(state) = &mut outbound_watch.state_mut() else {
                veilid_log!(self warn "watch being cancelled should have current state");
                return;
            };
            let mut per_node_states = vec![];
            let mut missing_pnks = BTreeSet::new();
            for pnk in state.nodes() {
                let Some(per_node_state) = inner
                    .outbound_watch_manager
                    .per_node_states
                    .get(pnk)
                    .cloned()
                else {
                    veilid_log!(self warn "missing per-node state for watch");
                    missing_pnks.insert(pnk.clone());
                    continue;
                };
                per_node_states.push((pnk.clone(), per_node_state));
            }

            state.edit(&inner.outbound_watch_manager.per_node_states, |editor| {
                editor.retain_nodes(|x| !missing_pnks.contains(x));
            });

            per_node_states
        };

        // Now reach out to each node and cancel their watch ids
        let mut unord = FuturesUnordered::new();
        for (pnk, pns) in per_node_states {
            let opaque_record_key = opaque_record_key.clone();
            unord.push(async move {
                let res = Box::pin(self.outbound_watch_value_cancel(
                    opaque_record_key,
                    pns.opt_watcher,
                    pns.safety_selection,
                    pns.watch_node_ref.unwrap(),
                    pns.watch_id,
                ))
                .await;
                (pnk, res)
            });
        }

        let mut cancelled = vec![];
        while let Some((pnk, res)) = unord.next().await {
            match res {
                Ok(_) => {
                    // Remove from 'per node states' because we got some response
                    cancelled.push(pnk);
                }
                Err(e) => {
                    veilid_log!(self debug "Outbound watch cancel error: {}", e);

                    // xxx should do something different for network unreachable vs host unreachable
                    // Leave in the 'per node states' for now because we couldn't contact the node
                    // but remove from this watch. We'll try the cancel again if we reach this node again during fanout.
                }
            }
        }

        // Update state
        {
            let mut inner = self.inner.lock();

            // Remove per node watches we cancelled
            for pnk in cancelled {
                if inner
                    .outbound_watch_manager
                    .per_node_states
                    .remove(&pnk)
                    .is_none()
                {
                    veilid_log!(self warn "per-node watch being cancelled should have still been in the table");
                };
            }

            // Remove outbound watch we've cancelled
            let Some(outbound_watch) = inner
                .outbound_watch_manager
                .outbound_watches
                .get_mut(&opaque_record_key)
            else {
                veilid_log!(self warn "watch being cancelled should have still been in the table");
                return;
            };

            // Mark as dead now that we cancelled
            outbound_watch.clear_state();
        }
    }

    /// See which existing per-node watches can be renewed
    /// and drop the ones that can't be or are dead
    pub(super) async fn process_outbound_watch_renew(
        &self,
        watch_lock: StorageManagerRecordLockGuard,
    ) {
        let opaque_record_key = watch_lock.record();

        // If we can't do this operation right now, don't try
        if !self.dht_is_online() {
            return;
        }

        let (record_key, per_node_states, per_node_params) = {
            let inner = &mut *self.inner.lock();
            let Some(outbound_watch) = inner
                .outbound_watch_manager
                .outbound_watches
                .get_mut(&opaque_record_key)
            else {
                veilid_log!(self warn "watch being renewed should have still been in the table");
                return;
            };

            let Some(desired) = outbound_watch.desired() else {
                veilid_log!(self warn "watch being renewed should have desired parameters");
                return;
            };

            let Some(state) = outbound_watch.state_mut() else {
                veilid_log!(self warn "watch being renewed should have current state");
                return;
            };

            let mut per_node_states = vec![];
            let mut missing_pnks = BTreeSet::new();
            for pnk in state.nodes() {
                let Some(per_node_state) = inner
                    .outbound_watch_manager
                    .per_node_states
                    .get(pnk)
                    .cloned()
                else {
                    veilid_log!(self warn "missing per-node state for watch");
                    missing_pnks.insert(pnk.clone());
                    continue;
                };
                per_node_states.push((pnk.clone(), per_node_state));
            }
            state.edit(&inner.outbound_watch_manager.per_node_states, |editor| {
                editor.retain_nodes(|x| !missing_pnks.contains(x));
            });

            let per_node_params = state.get_per_node_params(&desired);

            (
                outbound_watch.record_key(),
                per_node_states,
                per_node_params,
            )
        };

        // Now reach out to each node and renew their watches
        let mut unord = FuturesUnordered::new();
        for (_pnk, pns) in per_node_states {
            let params = per_node_params.clone();
            let opaque_record_key = opaque_record_key.clone();
            unord.push(async move {
                self.outbound_watch_value_change(
                    opaque_record_key,
                    params,
                    pns.watch_node_ref.unwrap(),
                    pns.watch_id,
                )
                .await
            });
        }

        // Process and merge all results since we're not fanning out
        let mut opt_owvresult: Option<OutboundWatchValueResult> = None;
        while let Some(res) = unord.next().await {
            match res {
                Ok(r) => {
                    opt_owvresult = match opt_owvresult {
                        Some(mut owvresult) => {
                            owvresult.merge(r);
                            Some(owvresult)
                        }
                        None => Some(r),
                    };
                }
                Err(e) => {
                    veilid_log!(self debug "Outbound watch change error: {}", e);
                }
            }
        }

        // Update state with merged results if we have them
        if let Some(owvresult) = opt_owvresult {
            let inner = &mut *self.inner.lock();
            self.process_outbound_watch_value_result_inner(inner, record_key, owvresult);
        }
    }

    /// Perform fanout to add or update per-node watches to an outbound watch
    pub(super) async fn process_outbound_watch_reconcile(
        &self,
        watch_lock: StorageManagerRecordLockGuard,
    ) {
        let opaque_record_key = watch_lock.record();

        // If we can't do this operation right now, don't try
        if !self.dht_is_online() {
            return;
        }

        // Get the nodes already active on this watch,
        // and the parameters to fanout with for the rest
        let (record_key, per_node_state, per_node_params) = {
            let inner = &mut *self.inner.lock();
            let Some(outbound_watch) = inner
                .outbound_watch_manager
                .outbound_watches
                .get_mut(&opaque_record_key)
            else {
                veilid_log!(self warn "watch being reconciled should have still been in the table");
                return;
            };

            let record_key = outbound_watch.record_key();

            // Get params to reconcile
            let Some(desired) = outbound_watch.desired() else {
                veilid_log!(self warn "watch being reconciled should have had desired parameters");
                return;
            };

            // Get active per node states
            let (mut per_node_state, per_node_params) = if let Some(state) = outbound_watch.state()
            {
                let per_node_state = state
                    .nodes()
                    .iter()
                    .map(|pnk| {
                        (
                            pnk.clone(),
                            inner
                                .outbound_watch_manager
                                .per_node_states
                                .get(pnk)
                                .cloned()
                                .unwrap(),
                        )
                    })
                    .collect();
                let per_node_params = state.get_per_node_params(&desired);

                (per_node_state, per_node_params)
            } else {
                (HashMap::new(), desired)
            };

            // Add in any inactive per node states
            for (pnk, pns) in &inner.outbound_watch_manager.per_node_states {
                // Skip any we have already
                if per_node_state.contains_key(pnk) {
                    continue;
                }
                // Add inactive per node state if the record key matches
                if pnk.record_key == record_key {
                    per_node_state.insert(pnk.clone(), pns.clone());
                }
            }

            (record_key, per_node_state, per_node_params)
        };

        // Now fan out with parameters and get new per node watches
        let cur_ts = Timestamp::now_non_decreasing();
        let res = self
            .outbound_watch_value(record_key.clone(), per_node_params, per_node_state)
            .await;

        {
            let inner = &mut *self.inner.lock();
            match res {
                Ok(owvresult) => {
                    // Update state
                    self.process_outbound_watch_value_result_inner(
                        inner,
                        record_key.clone(),
                        owvresult,
                    );

                    // If we succeeded update the last consensus node count
                    let Some(outbound_watch) = inner
                        .outbound_watch_manager
                        .outbound_watches
                        .get_mut(&opaque_record_key)
                    else {
                        veilid_log!(self warn "watch being reconciled should have still been in the table");
                        return;
                    };
                    let Some(state) = outbound_watch.state_mut() else {
                        veilid_log!(self warn "watch being reconciled should have had a state");
                        return;
                    };
                    state.edit(&inner.outbound_watch_manager.per_node_states, |editor| {
                        editor.update_last_consensus_node_count();
                    });
                }
                Err(e) => {
                    veilid_log!(self debug "Outbound watch fanout error: {}", e);
                }
            }

            // Regardless of result, set our next possible reconciliation time
            let next_ts = cur_ts.later(RECONCILE_OUTBOUND_WATCHES_INTERVAL);
            inner
                .outbound_watch_manager
                .set_next_reconcile_ts(opaque_record_key, next_ts);
        }
    }

    fn process_outbound_watch_value_result_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: RecordKey,
        owvresult: OutboundWatchValueResult,
    ) {
        let kind = record_key.kind();
        let opaque_record_key = record_key.opaque();

        let Some(outbound_watch) = inner
            .outbound_watch_manager
            .outbound_watches
            .get_mut(&opaque_record_key)
        else {
            veilid_log!(self warn "Outbound watch should have still been in the table");
            return;
        };
        let Some(desired) = outbound_watch.desired() else {
            veilid_log!(self warn "Watch with result should have desired params");
            return;
        };

        let watch_subkeys = desired.subkeys.clone();
        let opt_old_state_params = outbound_watch.state().map(|s| s.params().clone());

        let state = outbound_watch.state_mut_or_create(|| desired.clone());

        let mut added_nodes = Vec::new();
        let mut remove_nodes = BTreeSet::new();

        // Handle accepted
        for accepted_watch in owvresult.accepted {
            let node_id = accepted_watch.node_ref.node_ids().get(kind).unwrap();
            let pnk = PerNodeKey {
                record_key: record_key.clone(),
                node_id,
            };

            let expiration = accepted_watch.expiration;
            let count = state.remaining_count();

            // Check for accepted watch that came back with a dead watch
            // (non renewal, watch id didn't exist, didn't renew in time)
            if expiration.as_u64() != 0 && count > 0 {
                // Insert state, possibly overwriting an existing one
                let watch_id = accepted_watch.watch_id;
                let opt_watcher = desired.opt_watcher.clone();
                let safety_selection = desired.safety_selection.clone();
                let watch_node_ref = Some(accepted_watch.node_ref);
                let opt_value_changed_route = accepted_watch.opt_value_changed_route;

                inner.outbound_watch_manager.per_node_states.insert(
                    pnk.clone(),
                    OutboundWatchPerNodeState {
                        watch_id,
                        safety_selection,
                        opt_watcher,
                        expiration,
                        count,
                        watch_node_ref,
                        opt_value_changed_route,
                    },
                );
                added_nodes.push(pnk);
            } else {
                // Remove per node state because this watch id was not renewed
                inner.outbound_watch_manager.per_node_states.remove(&pnk);
                remove_nodes.insert(pnk);
            }
        }
        // Eliminate rejected
        for rejected_node_ref in owvresult.rejected {
            let node_id = rejected_node_ref.node_ids().get(kind).unwrap();
            let pnk = PerNodeKey {
                record_key: record_key.clone(),
                node_id,
            };
            inner.outbound_watch_manager.per_node_states.remove(&pnk);
            remove_nodes.insert(pnk);
        }
        // Drop unanswered but leave in per node state
        for ignored_node_ref in owvresult.ignored {
            let node_id = ignored_node_ref.node_ids().get(kind).unwrap();
            let pnk = PerNodeKey {
                record_key: record_key.clone(),
                node_id,
            };
            remove_nodes.insert(pnk);
        }

        // Update watch state
        let did_add_nodes = !added_nodes.is_empty();

        state.edit(&inner.outbound_watch_manager.per_node_states, |editor| {
            editor.set_params(desired.clone());
            editor.retain_nodes(|x| !remove_nodes.contains(x));
            editor.add_nodes(added_nodes);
        });

        // Watch was reconciled, now kick off an inspect to
        // ensure that any changes online are immediately reported to the app
        // If the watch parameters changed, or we added new nodes to the watch state
        // then we should inspect and see if anything changed
        if opt_old_state_params != Some(desired) || did_add_nodes {
            inner
                .outbound_watch_manager
                .enqueue_change_inspect(self, record_key, watch_subkeys);
        }
    }

    /// Get the next operation for a particular watch's state machine
    /// Can be processed in the foreground, or by the background operation queue
    pub(super) fn get_next_outbound_watch_operation(
        &self,
        key: RecordKey,
        cur_ts: Timestamp,
        outbound_watch: &mut OutboundWatch,
    ) -> Option<PinBoxFutureStatic<()>> {
        let registry = self.registry();
        let consensus_count = self.config().network.dht.get_value_count as usize;

        // Operate on this watch only if it isn't already being operated on
        let watch_lock = self
            .record_lock_table
            .try_lock_record(key.opaque(), StorageManagerRecordLockPurpose::Watch)?;

        // Terminate the 'desired' params for watches
        // that have no remaining count or have expired
        outbound_watch.try_expire_desired_state(cur_ts);

        // Check states
        if outbound_watch.is_dead() {
            // Outbound watch is dead
            let fut = {
                let registry = self.registry();
                async move {
                    registry
                        .storage_manager()
                        .process_outbound_watch_dead(watch_lock)
                }
            };
            return Some(pin_dyn_future!(fut));
        } else if outbound_watch.needs_cancel(&registry) {
            // Outbound watch needs to be cancelled
            let fut = {
                let registry = self.registry();
                async move {
                    registry
                        .storage_manager()
                        .process_outbound_watch_cancel(watch_lock)
                        .await
                }
            };
            return Some(pin_dyn_future!(fut));
        } else if outbound_watch.needs_renew(&registry, consensus_count, cur_ts) {
            // Outbound watch expired but can be renewed
            let fut = {
                let registry = self.registry();
                async move {
                    registry
                        .storage_manager()
                        .process_outbound_watch_renew(watch_lock)
                        .await
                }
            };
            return Some(pin_dyn_future!(fut));
        } else if outbound_watch.needs_reconcile(&registry, consensus_count, cur_ts) {
            // Outbound watch parameters have changed or it needs more nodes
            let fut = {
                let registry = self.registry();
                async move {
                    registry
                        .storage_manager()
                        .process_outbound_watch_reconcile(watch_lock)
                        .await
                }
            };
            return Some(pin_dyn_future!(fut));
        }
        None
    }

    /// Perform an inspection of the record's subkeys to see if we have the latest data
    /// If not, then get the first changed subkey and post a ValueChanged update about it
    /// Can be processed in the foreground, or by the background operation queue
    pub(super) fn get_change_inspection_operation(
        &self,
        record_key: RecordKey,
        subkeys: ValueSubkeyRangeSet,
    ) -> PinBoxFutureStatic<()> {
        let fut = {
            let opaque_record_key = record_key.opaque();
            let registry = self.registry();
            async move {
                let this = registry.storage_manager();

                let report = match this
                    .inspect_record(record_key.clone(), subkeys.clone(), DHTReportScope::SyncGet)
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        veilid_log!(this debug "Failed to inspect record for changes: {}", e);
                        return;
                    }
                };
                let mut newer_online_subkeys = report.newer_online_subkeys();

                // Get changed first changed subkey until we find one to report
                let mut n = 0;
                while !newer_online_subkeys.is_empty() {
                    let first_changed_subkey = newer_online_subkeys.first().unwrap();

                    let value = match this
                        .get_value(record_key.clone(), first_changed_subkey, true)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            veilid_log!(this debug "Failed to get changed record: {}", e);
                            return;
                        }
                    };

                    if let Some(value) = value {
                        let local_seq = report.local_seqs()[n];
                        if value.seq() > local_seq {
                            // Calculate the update
                            let (changed_subkeys, remaining_count, value) = {
                                let _watch_lock = this
                                    .record_lock_table
                                    .lock_record(
                                        opaque_record_key.clone(),
                                        StorageManagerRecordLockPurpose::Watch,
                                    )
                                    .await;

                                let inner = &mut *this.inner.lock();

                                // Get the outbound watch
                                let Some(outbound_watch) = inner
                                    .outbound_watch_manager
                                    .outbound_watches
                                    .get_mut(&opaque_record_key)
                                else {
                                    // No outbound watch means no callback
                                    return;
                                };

                                let Some(state) = outbound_watch.state_mut() else {
                                    // No outbound watch current state means no callback
                                    return;
                                };

                                // the remaining updates count
                                let remaining_count = state.remaining_count().saturating_sub(1);
                                state.edit(
                                    &inner.outbound_watch_manager.per_node_states,
                                    |editor| {
                                        editor.set_remaining_count(remaining_count);
                                    },
                                );

                                (newer_online_subkeys, remaining_count, value)
                            };

                            // Send the update
                            this.update_callback_value_change(
                                record_key,
                                changed_subkeys,
                                remaining_count,
                                Some(value),
                            );

                            // Update was sent, we're done
                            return;
                        }
                    }

                    // If we didn't send an update, remove the first changed subkey and try again
                    newer_online_subkeys.pop_first();
                    n += 1;
                }
            }
        };
        pin_dyn_future!(fut)
    }

    /// Handle a received 'Watch Value' query
    #[allow(clippy::too_many_arguments)]
    #[instrument(level = "trace", target = "dht", skip_all)]
    pub async fn inbound_watch_value(
        &self,
        opaque_record_key: OpaqueRecordKey,
        params: InboundWatchParameters,
        opt_raw_watch_id: Option<u64>,
    ) -> VeilidAPIResult<NetworkResult<InboundWatchValueResult>> {
        // Validate input
        if params.count == 0 && (opt_raw_watch_id.unwrap_or_default() == 0) {
            // Can't cancel a watch without a watch id
            return VeilidAPIResult::Ok(NetworkResult::invalid_message(
                "can't cancel watch without id",
            ));
        }

        let remote_record_store = self.get_remote_record_store()?;

        let opt_watch_id = match opt_raw_watch_id {
            Some(raw_watch_id) => {
                match remote_record_store.lookup_inbound_watch_id(raw_watch_id)? {
                    Some(id) => Some(id),
                    None => {
                        return Ok(NetworkResult::value(InboundWatchValueResult::Rejected));
                    }
                }
            }
            None => None,
        };

        if remote_record_store.contains_record(&opaque_record_key) {
            return remote_record_store
                .watch_record(opaque_record_key, params, opt_watch_id)
                .await
                .map(NetworkResult::value);
        }

        // No record found
        Ok(NetworkResult::value(InboundWatchValueResult::Rejected))
    }

    fn get_watched_subkeys_inner(
        &self,
        inner: &StorageManagerInner,
        record_key: &RecordKey,
    ) -> VeilidAPIResult<Option<ValueSubkeyRangeSet>> {
        let opaque_record_key = record_key.opaque();

        // Get the outbound watch
        let Some(outbound_watch) = inner
            .outbound_watch_manager
            .outbound_watches
            .get(&opaque_record_key)
        else {
            // No outbound watch means no callback
            return Ok(None);
        };

        let Some(desired) = outbound_watch.desired() else {
            // No outbound watch desired state means no callback (we are cancelling)
            return Ok(None);
        };

        Ok(Some(desired.subkeys))
    }

    fn update_watch_per_node_state_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: &RecordKey,
        count: u32,
        inbound_node_id: &NodeId,
        watch_id: u64,
    ) -> VeilidAPIResult<bool> {
        let opaque_record_key = record_key.opaque();

        // Get the outbound watch
        let Some(outbound_watch) = inner
            .outbound_watch_manager
            .outbound_watches
            .get(&opaque_record_key)
        else {
            // No outbound watch means no callback
            return Ok(false);
        };

        let Some(state) = outbound_watch.state() else {
            // No outbound watch current state means no callback (we haven't reconciled yet)
            return Ok(false);
        };

        // If the reporting node is not part of our current watch, don't process the value change
        let pnk = PerNodeKey {
            record_key: record_key.clone(),
            node_id: inbound_node_id.clone(),
        };
        if !state.nodes().contains(&pnk) {
            return Ok(false);
        }

        // Get per node state
        let Some(per_node_state) = inner.outbound_watch_manager.per_node_states.get_mut(&pnk)
        else {
            // No per node state means no callback
            veilid_log!(self warn "Missing per node state in outbound watch: {:?}", pnk);
            return Ok(false);
        };

        // If watch id doesn't match it's for an older watch and should be ignored
        if per_node_state.watch_id != watch_id {
            // No per node state means no callback
            veilid_log!(self debug "Incorrect watch id for per node state in outbound watch: {:?} {} != {}", pnk, per_node_state.watch_id, watch_id);
            return Ok(false);
        }

        // Update per node state
        if count > per_node_state.count {
            // If count is greater than our requested count then this is invalid, cancel the watch
            // XXX: Should this be a punishment?
            veilid_log!(self debug
                "Watch count went backward: {} @ {} id={}: {} > {}",
                record_key,
                inbound_node_id,
                watch_id,
                count,
                per_node_state.count
            );

            // Force count to zero for this node id so it gets cancelled out by the background process
            per_node_state.count = 0;
            return Ok(false);
        } else if count == per_node_state.count {
            // If a packet gets delivered twice or something else is wrong, report a non-decremented watch count
            // Log this because watch counts should always be decrementing non a per-node basis.
            // XXX: Should this be a punishment?
            veilid_log!(self debug
                "Watch count duplicate: {} @ {} id={}: {} == {}",
                record_key,
                inbound_node_id,
                watch_id,
                count,
                per_node_state.count
            );
        } else {
            // Reduce the per-node watch count
            veilid_log!(self debug
                "Watch count decremented: {} @ {} id={}: {} < {}",
                record_key,
                inbound_node_id,
                watch_id,
                count,
                per_node_state.count
            );
            per_node_state.count = count;
        }

        Ok(true)
    }

    async fn accept_single_value_change_and_report(
        &self,
        record_lock: StorageManagerRecordLockGuard,
        record_key: RecordKey,
        reportable_subkeys: ValueSubkeyRangeSet,
        value: Arc<SignedValueData>,
    ) -> VeilidAPIResult<()> {
        let opaque_record_key = record_lock.record();

        // Set the local value
        self.handle_set_single_local_value_with_single_record_lock(
            &record_lock,
            reportable_subkeys.first().unwrap(),
            value.clone(),
        )
        .await?;

        // Update the watch state if we're going to report
        let remaining_count = {
            let inner = &mut *self.inner.lock();

            // Get the outbound watch
            if let Some(outbound_watch) = inner
                .outbound_watch_manager
                .outbound_watches
                .get_mut(&opaque_record_key)
            {
                if let Some(state) = outbound_watch.state_mut() {
                    // If we're going to report, update the remaining change count
                    let remaining_count = state.remaining_count().saturating_sub(1);
                    state.edit(&inner.outbound_watch_manager.per_node_states, |editor| {
                        editor.set_remaining_count(remaining_count);
                    });

                    state.remaining_count()
                } else {
                    // No outbound watch current state means watch died (no remaining count)
                    0
                }
            } else {
                // No outbound watch means watch died (no remaining count)
                0
            }
        };

        // Announce ValueChanged VeilidUpdate
        // Cancellations (count=0) are sent by process_outbound_watch_dead(), not here
        let value = self.maybe_decrypt_value_data(&record_key, value.value_data())?;

        drop(record_lock);

        // We have a single value with a newer sequence number to report
        self.update_callback_value_change(
            record_key,
            reportable_subkeys,
            remaining_count,
            Some(value),
        );

        Ok(())
    }

    /// Handle a received 'Value Changed' statement
    #[instrument(level = "debug", target = "watch", skip_all)]
    pub async fn inbound_value_changed(
        &self,
        opaque_record_key: OpaqueRecordKey,
        subkeys: ValueSubkeyRangeSet,
        count: u32,
        mut opt_value: Option<Arc<SignedValueData>>,
        inbound_node_id: NodeId,
        watch_id: u64,
    ) -> VeilidAPIResult<NetworkResult<()>> {
        let Ok(record_key) = self.get_record_key_for_opaque_record_key(&opaque_record_key) else {
            // value change received for unopened key
            return Ok(NetworkResult::value(()));
        };

        // Operate on the watch for this record
        let record_lock = self
            .record_lock_table
            .lock_record(record_key.opaque(), StorageManagerRecordLockPurpose::Watch)
            .await;

        // Determine if we have reportable subkeys
        // Also clears the value if the value should not be reported
        let mut reportable_subkeys = {
            let inner = &mut *self.inner.lock();

            // No subkeys means remote node cancelled, but we already captured that with the
            // assignment of 'count' to the per_node_state above, so we can just jump out here
            let Some(first_subkey) = subkeys.first() else {
                return Ok(NetworkResult::value(()));
            };

            // If more than one subkey was reported then ignore any value because we want
            // to do all our watch updates transactionally and we'll just kick off a change inspection
            if subkeys.len() > 1 {
                opt_value = None;
            }

            // Get what subkeys are being watched
            let Some(watched_subkeys) = self.get_watched_subkeys_inner(inner, &record_key)? else {
                // Nothing watched, nothing to report
                return Ok(NetworkResult::value(()));
            };

            // Update the per-node watch state and determine if this watch should be be reported
            let watch_change_accepted = self.update_watch_per_node_state_inner(
                inner,
                &record_key,
                count,
                &inbound_node_id,
                watch_id,
            )?;
            if !watch_change_accepted {
                // Per-node state update was rejected, skip this
                return Ok(NetworkResult::value(()));
            }

            // If our watched subkey range differs from the reported change's range
            // we should only report changes that we care about
            let reportable_subkeys = subkeys.intersect(&watched_subkeys);
            if let Some(first_reportable_subkey) = reportable_subkeys.first() {
                if first_reportable_subkey != first_subkey {
                    opt_value = None;
                }
            } else {
                opt_value = None;
            }

            reportable_subkeys
        };

        // See if the value we received can be reported as-is
        if let Some(value) = opt_value.clone() {
            // Safe to unwrap here because if we have reportable subkeys, there must be
            // a first reportable subkey, and if not, the value is set to None
            let first_subkey = reportable_subkeys.first().unwrap();

            let last_get_result = self
                .handle_get_single_local_value(&opaque_record_key, first_subkey, true)
                .await?;

            let descriptor = last_get_result.opt_descriptor.unwrap();
            let schema = descriptor.schema()?;

            // Validate with schema
            if self
                .check_subkey_value_data(
                    &schema,
                    descriptor.ref_owner(),
                    first_subkey,
                    value.value_data(),
                )
                .is_err()
            {
                // Validation failed, ignore this value
                // Move to the next node
                return Ok(NetworkResult::invalid_message(format!(
                    "Schema validation failed on subkey {}",
                    first_subkey
                )));
            }

            // Make sure this value would actually be newer
            if let Some(last_value) = &last_get_result.opt_value {
                if value.value_data().seq() <= last_value.value_data().seq() {
                    // inbound value is older than or equal to the sequence number that we have
                    // so we're not going to report this
                    opt_value = None;

                    // Shrink up the subkey range because we're removing the first value from the things we'd possibly report on
                    reportable_subkeys.pop_first().unwrap();
                    if reportable_subkeys.is_empty() {
                        // If there's nothing left to report, just return no
                        return Ok(NetworkResult::value(()));
                    }
                }
            }
        }

        // Keep the value because it is newer than the one we have, and it was the only
        // value we got a change update for. Report the change as well to the app.
        if let Some(value) = opt_value.clone() {
            self.accept_single_value_change_and_report(
                record_lock,
                record_key,
                reportable_subkeys,
                value,
            )
            .await?;
        } else if !reportable_subkeys.is_empty() {
            // We have subkeys that have be reported as possibly changed
            // but not a specific record reported, so we should defer reporting and
            // inspect the range to see what changed

            // Queue this up for inspection
            let mut inner = self.inner.lock();
            inner.outbound_watch_manager.enqueue_change_inspect(
                self,
                record_key,
                reportable_subkeys,
            );
        }

        Ok(NetworkResult::value(()))
    }

    /// Check all watches for changes
    /// Used when we come back online from being offline and may have
    /// missed some ValueChanged notifications
    #[instrument(level = "trace", target = "watch", skip_all)]
    pub fn change_inspect_all_watches(&self) {
        let mut inner = self.inner.lock();

        let mut change_inspects = vec![];
        for outbound_watch in inner.outbound_watch_manager.outbound_watches.values() {
            if let Some(state) = outbound_watch.state() {
                let reportable_subkeys = state.params().subkeys.clone();
                change_inspects.push((outbound_watch.record_key(), reportable_subkeys));
            }
        }

        if change_inspects.is_empty() {
            return;
        }

        veilid_log!(self debug "change inspecting {} watches", change_inspects.len());

        for change_inspect in change_inspects {
            inner.outbound_watch_manager.enqueue_change_inspect(
                self,
                change_inspect.0,
                change_inspect.1,
            );
        }
    }
}
