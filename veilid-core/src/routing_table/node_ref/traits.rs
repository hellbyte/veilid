use super::*;

// Field accessors
pub(crate) trait NodeRefAccessorsTrait {
    fn entry(&self) -> Arc<BucketEntry>;
    fn sequencing(&self) -> Sequencing;
    fn routing_domain_set(&self) -> RoutingDomainSet;
    fn filter(&self) -> NodeRefFilter;
    fn take_filter(&mut self) -> NodeRefFilter;
    fn dial_info_filter(&self) -> DialInfoFilter;

    // fn node_info_outbound_filter(&self, routing_domain: RoutingDomain) -> DialInfoFilter;
    // fn is_filter_dead(&self) -> bool;
}

// Operate on entry
pub(crate) trait NodeRefOperateTrait {
    fn operate<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&RoutingTableInner, &BucketEntryInner) -> T;
    fn operate_mut<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut RoutingTableInner, &mut BucketEntryInner) -> T;
}

// Common Operations
pub(crate) trait NodeRefCommonTrait: NodeRefAccessorsTrait + NodeRefOperateTrait {
    fn same_entry<T: NodeRefAccessorsTrait + ?Sized>(&self, other: &T) -> bool {
        Arc::ptr_eq(&self.entry(), &other.entry())
    }

    fn same_bucket_entry(&self, entry: &Arc<BucketEntry>) -> bool {
        Arc::ptr_eq(&self.entry(), entry)
    }

    fn equivalent<T: NodeRefAccessorsTrait + ?Sized>(&self, other: &T) -> bool {
        self.same_entry(other)
            && self.filter() == other.filter()
            && self.sequencing() == other.sequencing()
    }

    fn node_ids(&self) -> NodeIdGroup {
        self.operate(|_rti, e| e.node_ids())
    }
    fn public_keys(&self, routing_domain: RoutingDomain) -> PublicKeyGroup {
        self.operate(|_rti, e| e.public_keys(routing_domain))
    }
    fn best_node_id(&self) -> Option<NodeId> {
        self.operate(|_rti, e| e.best_node_id())
    }
    fn best_public_key(&self, routing_domain: RoutingDomain) -> Option<PublicKey> {
        self.operate(|_rti, e| e.best_public_key(routing_domain))
    }

    fn relay_ids(&self, routing_domain: RoutingDomain) -> Vec<NodeIdGroup> {
        self.operate(|_rti, e| e.relay_ids(routing_domain))
    }

    fn update_node_status(&self, routing_domain: RoutingDomain, node_status: NodeStatus) {
        self.operate_mut(|_rti, e| {
            e.update_node_status(routing_domain, node_status);
        });
    }
    fn best_routing_domain(&self) -> Option<RoutingDomain> {
        self.operate(|rti, e| e.best_routing_domain(rti, self.routing_domain_set()))
    }

    // fn envelope_support(&self) -> Vec<u8> {
    //     self.operate(|_rti, e| e.envelope_support())
    // }
    fn add_envelope_version(&self, envelope_version: EnvelopeVersion) {
        self.operate_mut(|_rti, e| e.add_envelope_version(envelope_version))
    }
    // fn set_envelope_support(&self, envelope_support: Vec<u8>) {
    //     self.operate_mut(|_rti, e| e.set_envelope_support(envelope_support))
    // }
    fn best_envelope_version(&self) -> Option<EnvelopeVersion> {
        self.operate(|_rti, e| e.best_envelope_version())
    }
    fn state_reason(&self, cur_ts: Timestamp) -> BucketEntryStateReason {
        self.operate(|_rti, e| e.state_reason(cur_ts))
    }
    fn state(&self, cur_ts: Timestamp) -> BucketEntryState {
        self.operate(|_rti, e| e.state(cur_ts))
    }
    fn peer_stats(&self) -> PeerStats {
        self.operate(|_rti, e| e.peer_stats().clone())
    }

    fn get_peer_info(&self, routing_domain: RoutingDomain) -> Option<Arc<PeerInfo>> {
        self.operate(|_rti, e| e.get_peer_info(routing_domain))
    }
    fn node_info(&self, routing_domain: RoutingDomain) -> Option<NodeInfo> {
        self.operate(|_rti, e| e.node_info(routing_domain).cloned())
    }
    fn peer_info_has_valid_signature(&self, routing_domain: RoutingDomain) -> bool {
        self.operate(|_rti, e| {
            e.get_peer_info(routing_domain)
                .map(|pi| !pi.signatures().is_empty())
                .unwrap_or(false)
        })
    }
    fn node_info_ts(&self, routing_domain: RoutingDomain) -> Timestamp {
        self.operate(|_rti, e| {
            e.node_info(routing_domain)
                .map(|ni| ni.timestamp())
                .unwrap_or(0u64.into())
        })
    }
    fn has_seen_our_node_info_ts(&self, routing_domain: RoutingDomain) -> bool {
        self.operate(|rti, e| {
            let Some(our_node_info_ts) = rti
                .get_published_peer_info(routing_domain)
                .map(|pi| pi.node_info().timestamp())
            else {
                return false;
            };
            e.has_seen_our_node_info_ts(routing_domain, our_node_info_ts)
        })
    }
    fn set_seen_our_node_info_ts(
        &self,
        routing_domain: RoutingDomain,
        seen_ts: Timestamp,
    ) -> Option<Timestamp> {
        self.operate_mut(|_rti, e| e.set_seen_our_node_info_ts(routing_domain, seen_ts))
    }
    // fn outbound_protocols(&self, routing_domain: RoutingDomain) -> Option<ProtocolTypeSet> {
    //     self.operate(|_rt, e| e.node_info(routing_domain).map(|n| n.outbound_protocols()))
    // }
    // fn address_types(&self, routing_domain: RoutingDomain) -> Option<AddressTypeSet> {
    //     self.operate(|_rt, e| e.node_info(routing_domain).map(|n| n.address_types()))
    // }

    // DialInfo
    fn first_dial_info_detail(&self) -> Option<DialInfoDetail> {
        let routing_domain_set = self.routing_domain_set();
        let dial_info_filter = self.dial_info_filter();
        let sequencing = self.sequencing();
        let (ordering, dial_info_filter) = dial_info_filter.apply_sequencing(sequencing);
        let sort = DialInfoDetail::get_ordering_sort(ordering);

        if dial_info_filter.is_dead() {
            return None;
        }

        let filter = |did: &DialInfoDetail| did.matches_filter(&dial_info_filter);

        self.operate(|_rt, e| {
            for routing_domain in routing_domain_set {
                if let Some(ni) = e.node_info(routing_domain) {
                    if let Some(did) = ni.first_filtered_dial_info_detail(sort.as_deref(), &filter)
                    {
                        return Some(did);
                    }
                }
            }
            None
        })
    }

    fn dial_info_details(&self) -> Vec<DialInfoDetail> {
        let routing_domain_set = self.routing_domain_set();
        let dial_info_filter = self.dial_info_filter();
        let sequencing = self.sequencing();
        let (ordering, dial_info_filter) = dial_info_filter.apply_sequencing(sequencing);
        let sort = DialInfoDetail::get_ordering_sort(ordering);

        let mut out = Vec::new();

        if dial_info_filter.is_dead() {
            return out;
        }

        let filter = |did: &DialInfoDetail| did.matches_filter(&dial_info_filter);

        self.operate(|_rt, e| {
            for routing_domain in routing_domain_set {
                if let Some(ni) = e.node_info(routing_domain) {
                    let mut dids = ni.filtered_dial_info_details(sort.as_deref(), &filter);
                    out.append(&mut dids);
                }
            }
        });
        out.remove_duplicates();
        out
    }

    /// Get the most recent 'last connection' to this node matching the node ref filter
    /// Filtered first and then sorted by ordering preference and then by most recent
    fn last_flow(&self) -> Option<Flow> {
        self.operate(|rti, e| {
            // apply sequencing to filter and get sort
            let sequencing = self.sequencing();
            let filter = self.filter();
            let (ordering, filter) = filter.apply_sequencing(sequencing);
            let mut last_flows = e.last_flows(rti, true, filter);

            if let Some(sort) = ProtocolType::get_ordering_sort(ordering) {
                last_flows.sort_by(|a, b| sort(&a.0.protocol_type(), &b.0.protocol_type()));
            }

            last_flows.first().map(|x| x.0)
        })
    }

    /// Get all the 'last connection' flows for this node matching the node ref filter
    /// Filtered first and then sorted by ordering preference and then by most recent
    #[expect(dead_code)]
    fn last_flows(&self) -> Vec<Flow> {
        self.operate(|rti, e| {
            // apply sequencing to filter and get sort
            let sequencing = self.sequencing();
            let filter = self.filter();
            let (ordering, filter) = filter.apply_sequencing(sequencing);
            let mut last_flows = e.last_flows(rti, true, filter);

            if let Some(sort) = ProtocolType::get_ordering_sort(ordering) {
                last_flows.sort_by(|a, b| sort(&a.0.protocol_type(), &b.0.protocol_type()));
            }

            last_flows.into_iter().map(|x| x.0).collect()
        })
    }

    fn clear_last_flows(&self) {
        self.operate_mut(|_rti, e| e.clear_last_flows(self.dial_info_filter()))
    }

    fn set_last_flow(&self, flow: Flow, ts: Timestamp) {
        self.operate_mut(|rti, e| {
            e.set_last_flow(flow, ts);
            if let Some(best_node_id) = e.best_node_id() {
                rti.touch_recent_peer(best_node_id, flow);
            }
        })
    }

    fn clear_last_flow(&self, flow: Flow) {
        self.operate_mut(|_rti, e| {
            e.remove_last_flow(flow);
        })
    }

    fn is_relaying(&self, routing_domain: RoutingDomain) -> bool {
        self.operate(|rti, e| {
            let Some(relay_ids) = e
                .node_info(routing_domain)
                .map(|node_info| node_info.relay_ids())
            else {
                return false;
            };
            let our_node_ids = rti.routing_table().node_ids();
            our_node_ids.contains_any_from_slice(relay_ids.as_slice())
        })
    }

    fn has_any_dial_info(&self) -> bool {
        self.operate(|_rti, e| {
            for rtd in RoutingDomain::all() {
                if let Some(ni) = e.node_info(rtd) {
                    if ni.has_any_dial_info() {
                        return true;
                    }
                }
            }
            false
        })
    }

    fn report_protected_connection_dropped(&self) {
        self.stats_failed_to_send(Timestamp::now_non_decreasing(), false);
    }

    fn report_failed_route_test(&self) {
        self.stats_failed_to_send(Timestamp::now_non_decreasing(), false);
    }

    fn stats_question_sent(
        &self,
        ts: Timestamp,
        bytes: ByteCount,
        expects_answer: bool,
        ordering: SequenceOrdering,
    ) {
        self.operate_mut(|rti, e| {
            rti.transfer_stats_accounting().add_up(bytes);
            e.question_sent(ts, bytes, expects_answer, ordering);
        })
    }
    fn stats_question_rcvd(&self, ts: Timestamp, bytes: ByteCount) {
        self.operate_mut(|rti, e| {
            rti.transfer_stats_accounting().add_down(bytes);
            e.question_rcvd(ts, bytes);
        })
    }
    fn stats_answer_sent(&self, bytes: ByteCount) {
        self.operate_mut(|rti, e| {
            rti.transfer_stats_accounting().add_up(bytes);
            e.answer_sent(bytes);
        })
    }
    fn stats_answer_rcvd(
        &self,
        send_ts: Timestamp,
        recv_ts: Timestamp,
        bytes: ByteCount,
        ordering: SequenceOrdering,
    ) {
        self.operate_mut(|rti, e| {
            rti.transfer_stats_accounting().add_down(bytes);
            rti.latency_stats_accounting()
                .record_latency(recv_ts.duration_since(send_ts));
            e.answer_rcvd(send_ts, recv_ts, bytes, ordering);
        })
    }
    fn stats_lost_answer(&self, ordering: SequenceOrdering) {
        self.operate_mut(|_rti, e| {
            e.lost_answer(ordering);
        })
    }
    fn stats_failed_to_send(&self, ts: Timestamp, expects_answer: bool) {
        self.operate_mut(|_rti, e| {
            e.failed_to_send(ts, expects_answer);
        })
    }
    fn report_sender_info(
        &self,
        routing_domain: RoutingDomain,
        protocol_type: ProtocolType,
        address_type: AddressType,
        sender_info: SenderInfo,
    ) -> Option<SenderInfo> {
        self.operate_mut(|_rti, e| {
            e.report_sender_info(
                LastSenderInfoKey(routing_domain, protocol_type, address_type),
                sender_info,
            )
        })
    }
}
