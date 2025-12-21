use super::*;

impl_veilid_log_facility!("rtab");

impl RoutingTable {
    fn check_relay_valid(
        &self,
        cur_ts: Timestamp,
        relay: &RoutingDomainRelay,
        mut relay_state: RoutingDomainRelayState,
        relay_node_filter: &impl Fn(&BucketEntryInner) -> bool,
    ) -> Option<RoutingDomainRelayState> {
        let inner = self.inner.read();
        let rti = &inner;
        let locked_relay_node = relay.relay_node.locked(rti);

        let state_reason = locked_relay_node.state_reason(cur_ts);

        // No best node id
        let Some(relay_node_id) = locked_relay_node.best_node_id() else {
            veilid_log!(self debug "Relay node no longer has best node id, dropping relay {}", locked_relay_node);
            return None;
        };

        // Relay node is dead or no longer needed
        if matches!(
            state_reason,
            BucketEntryStateReason::Dead(_) | BucketEntryStateReason::Punished(_)
        ) {
            veilid_log!(self debug "Relay node is now {:?}, dropping relay {}", state_reason, locked_relay_node);
            return None;
        }

        // Relay node no longer can relay
        if locked_relay_node.operate(|_rti, e| !&relay_node_filter(e)) {
            veilid_log!(self debug
                "Relay node can no longer relay, dropping relay {}",
                relay.relay_node
            );
            return None;
        }

        // See if our relay was optimized last long enough ago to consider getting a new one
        // if it is no longer fast enough
        let last_optimized_duration = cur_ts.duration_since(relay_state.last_optimized);
        if last_optimized_duration > RELAY_OPTIMIZATION_INTERVAL {
            // See what our relay's current percentile is
            if let Some(relay_relative_performance) = inner.get_node_relative_performance(
                relay_node_id,
                cur_ts,
                relay_node_filter,
                |ls| ls.tm90,
            ) {
                // Get latency numbers
                let latency_stats = if let Some(latency) = locked_relay_node.peer_stats().latency {
                    latency.to_string()
                } else {
                    "[no stats]".to_owned()
                };

                // Get current relay reliability
                let state_reason = locked_relay_node.state_reason(cur_ts);

                if relay_relative_performance.percentile < RELAY_OPTIMIZATION_PERCENTILE {
                    // Drop the current relay so we can get the best new one
                    veilid_log!(self debug
                        "Relay tm90 is ({:.2}% < {:.2}%) ({} out of {}) (latency {}, {:?}) optimizing relay {}",
                        relay_relative_performance.percentile,
                        RELAY_OPTIMIZATION_PERCENTILE,
                        relay_relative_performance.node_index,
                        relay_relative_performance.node_count,
                        latency_stats,
                        state_reason,
                        locked_relay_node
                    );
                    return None;
                } else {
                    // Note that we tried to optimize the relay but it was good
                    veilid_log!(self debug
                        "Relay tm90 is ({:.2}% >= {:.2}%) ({} out of {}) (latency {}, {:?}) keeping {}",
                        relay_relative_performance.percentile,
                        RELAY_OPTIMIZATION_PERCENTILE,
                        relay_relative_performance.node_index,
                        relay_relative_performance.node_count,
                        latency_stats,
                        state_reason,
                        locked_relay_node
                    );
                    relay_state.last_optimized = cur_ts;
                }
            } else {
                // Drop the current relay because it could not be measured
                veilid_log!(self debug
                    "Relay relative performance not found {}",
                    locked_relay_node
                );
                return None;
            }
        }

        Some(relay_state)
    }

    // Keep relays assigned and accessible
    #[instrument(level = "trace", skip_all, err)]
    pub async fn relay_management_task_routine(
        &self,
        _stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        // Only do this if we've reached the state where we can allocate relays meaningfully
        let rds = self.routing_domain_state(RoutingDomain::PublicInternet);
        let mut relay_status = match rds {
            RoutingDomainState::Invalid
            | RoutingDomainState::NeedsDialInfoConfirmation
            | RoutingDomainState::Unusable => return Ok(()),
            RoutingDomainState::NeedsRelays { relay_status }
            | RoutingDomainState::ReadyToPublish { relay_status } => relay_status,
        };

        // Get existing relays
        let original_relays_and_states = self.relays_and_states(RoutingDomain::PublicInternet);

        // Validate existing relays
        let mut original_relays = vec![];
        let mut valid_relays = vec![];
        let mut state_updates = vec![];
        let relay_node_filter = self.make_public_internet_relay_node_filter();
        for (relay, relay_state) in original_relays_and_states {
            if let Some(updated_relay_state) =
                self.check_relay_valid(cur_ts, &relay, relay_state, &relay_node_filter)
            {
                if updated_relay_state != relay_state {
                    state_updates.push((relay.clone(), updated_relay_state));
                }
                valid_relays.push(relay.clone());
            }
            original_relays.push(relay);
        }

        // Apply new relay states if they have changed
        if !state_updates.is_empty() {
            let mut editor = self.edit_public_internet_routing_domain();
            for (relay, state) in state_updates {
                editor.set_relay_state(relay, state);
            }
            editor.commit(false).await;
        }

        // Drop outbound relay if it changed
        let mut has_outbound_relay = false;
        if let Some(outbound_relay_peerinfo) =
            intf::get_outbound_relay_peer(RoutingDomain::PublicInternet).await
        {
            valid_relays.retain(|rdr| {
                if matches!(rdr.relay_kind, RelayKind::Outbound) {
                    if rdr
                        .relay_node
                        .node_ids()
                        .contains_any_from_slice(outbound_relay_peerinfo.node_ids())
                    {
                        has_outbound_relay = true;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            });
        }

        // Allocate outbound relay if one is needed
        let mut attempted_relays = HashSet::<NodeId>::new();
        if let Some(outbound_relay_peerinfo) =
            intf::get_outbound_relay_peer(RoutingDomain::PublicInternet).await
        {
            if !has_outbound_relay {
                // Register new outbound relay
                match self.register_node_with_peer_info(outbound_relay_peerinfo, false) {
                    Ok(relay_node) => {
                        let outbound_relay = RoutingDomainRelay::new(
                            RoutingDomain::PublicInternet,
                            relay_node.unfiltered(),
                            RelayKind::Outbound,
                        );
                        for rid in outbound_relay.relay_node.node_ids().iter() {
                            attempted_relays.insert(rid.clone());
                        }
                        relay_status.apply_relay(outbound_relay);
                    }
                    Err(e) => {
                        veilid_log!(self error "failed to register outbound relay with peer info: {}", e);
                    }
                };
            }
        }

        // Apply valid inbound relays to status
        for relay in valid_relays {
            for rid in relay.relay_node.node_ids().iter() {
                attempted_relays.insert(rid.clone());
            }
            relay_status.apply_relay(relay);
        }

        // Allocate new inbound relays as needed
        while relay_status.wants_more_relays() {
            let next_relay_node_filter = |e: &BucketEntryInner| {
                // Exclude any relays we have already
                for nid in e.node_ids().iter() {
                    if attempted_relays.contains(nid) {
                        return false;
                    }
                }
                relay_node_filter(e)
            };
            // Find a node in our routing table that is an acceptable inbound relay
            if let Some(relay_node) = self.find_random_fast_node(
                cur_ts,
                next_relay_node_filter,
                RELAY_SELECTION_PERCENTILE,
                |ls| ls.tm90,
            ) {
                for rid in relay_node.node_ids().iter() {
                    attempted_relays.insert(rid.clone());
                }

                let routing_domain_relay = RoutingDomainRelay::new(
                    RoutingDomain::PublicInternet,
                    relay_node,
                    RelayKind::Inbound,
                );

                relay_status.apply_relay(routing_domain_relay);
            } else {
                // No relays left, we did our best
                break;
            }
        }

        // Get final sorted list of relays
        let new_relays = relay_status.get_sorted_relays_list();

        // Apply new relay list if it has changed
        if new_relays != original_relays {
            let mut editor = self.edit_public_internet_routing_domain();

            editor.set_relays(new_relays);

            // Commit the changes
            if editor.commit(false).await {
                // Try to publish the peer info
                editor.publish();
            }
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub fn make_public_internet_relay_node_filter(&self) -> impl Fn(&BucketEntryInner) -> bool {
        let ip6_prefix_size = self.config().network.max_connections_per_ip6_prefix_size as usize;

        // Get all our outbound protocol/address types
        let outbound_dif = self.get_outbound_dial_info_filter(RoutingDomain::PublicInternet);

        // Get our own peer info
        let own_peer_info = self.get_current_peer_info(RoutingDomain::PublicInternet);

        move |e: &BucketEntryInner| {
            // Ensure this node is not on the local network and is on the public internet
            if e.has_node_info(RoutingDomain::LocalNetwork.into()) {
                return false;
            }

            // Exclude any nodes that don't have a 'best node id' for our enabled cryptosystems
            if e.best_node_id().is_none() {
                return false;
            }

            // Exclude any nodes that have 'failed to send' state indicating a
            // connection drop or inability to reach the node
            if e.peer_stats().rpc_stats.failed_to_send > 0 {
                return false;
            }

            // Get the public internet peer info so we can validate it
            let Some(peer_info) = e.get_peer_info(RoutingDomain::PublicInternet) else {
                return false;
            };

            // Exclude any nodes that are relaying directly through us
            if own_peer_info
                .node_ids()
                .contains_any_from_slice(&peer_info.node_info().relay_ids())
            {
                return false;
            }

            // Disqualify nodes that don't have relay capability
            if !peer_info
                .node_info()
                .has_capability(VEILID_CAPABILITY_RELAY)
            {
                return false;
            }

            // Disqualify any nodes that don't speak all of the envelope versions we do
            let peer_envelope_support = peer_info.node_info().envelope_support();
            if own_peer_info
                .node_info()
                .envelope_support()
                .iter()
                .copied()
                .any(|x| !peer_envelope_support.contains(&x))
            {
                return false;
            }
            // Note: as of right now we don't need the relays to speak the same cryptography we do. Relays don't validate envelopes, they just forward them if they can.
            // If this changes, we may want to have relays match our crypto kinds too.
            // let peer_crypto_kinds = peer_info
            //     .node_info()
            //     .crypto_info_list()
            //     .iter()
            //     .map(|x| x.kind())
            //     .collect::<HashSet<_>>();
            // if own_peer_info
            //     .node_info()
            //     .crypto_info_list()
            //     .iter()
            //     .find(|x| !peer_crypto_kinds.contains(&x.kind()))
            //     .is_some()
            // {
            //     return false;
            // }

            // Ensure there is a way to reach this relay
            let mut directly_reachable = false;
            for did in peer_info.node_info().dial_info_detail_list() {
                if did.class.requires_signal() {
                    continue;
                }
                // If this dial info can be contacted directly, then it is a relay candidate
                if did.dial_info.matches_filter(&outbound_dif) {
                    directly_reachable = true;
                    break;
                }
            }

            if !directly_reachable {
                return false;
            }

            // Exclude any nodes that have our same network block
            if own_peer_info
                .node_info()
                .is_on_same_ipblock(peer_info.node_info(), ip6_prefix_size)
            {
                return false;
            }

            true
        }
    }
}
