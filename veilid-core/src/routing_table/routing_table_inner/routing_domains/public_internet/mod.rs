mod editor;

pub use editor::*;

use super::*;

impl_veilid_log_facility!("rtab");

/// Context for calculating contact method
struct ContactMethodContext<'a> {
    peer_a: Arc<PeerInfo>,
    peer_b: Arc<PeerInfo>,
    node_a: &'a NodeInfo,
    node_b: &'a NodeInfo,
    dial_info_filter: DialInfoFilter,
    sequencing: Sequencing,
    context_sort: Option<&'a DialInfoDetailSort<'a>>,
    best_ck: CryptoKind,
    same_ipblock: bool,
    //node_a_id: NodeId,
    node_b_id: NodeId,
}
impl fmt::Debug for ContactMethodContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContactMethodContext")
            .field("peer_a", &self.peer_a)
            .field("peer_b", &self.peer_b)
            .field("node_a", &self.node_a)
            .field("node_b", &self.node_b)
            .field("dial_info_filter", &self.dial_info_filter)
            .field("sequencing", &self.sequencing)
            //.field("context_sort", &self.context_sort)
            .field("best_ck", &self.best_ck)
            .field("same_ipblock", &self.same_ipblock)
            .field("node_b_id", &self.node_b_id)
            .finish()
    }
}

/// Public Internet routing domain internals
pub struct PublicInternetRoutingDomainDetail {
    /// Registry accessor
    registry: VeilidComponentRegistry,
    /// The interface networks that are in this domain
    interface_addresses: Vec<IfAddr>,
    /// Common implementation for all routing domains
    common: RoutingDomainDetailCommon,
    /// Published peer info for this routing domain
    published_peer_info: Mutex<Option<Arc<PeerInfo>>>,
}

impl fmt::Debug for PublicInternetRoutingDomainDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicInternetRoutingDomainDetail")
            // .field("registry", &self.registry)
            .field("common", &self.common)
            .field("published_peer_info", &self.published_peer_info)
            .finish()
    }
}

impl_veilid_component_accessors!(PublicInternetRoutingDomainDetail);

impl RoutingDomainDetailCommonAccessors for PublicInternetRoutingDomainDetail {
    fn common(&self) -> &RoutingDomainDetailCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut RoutingDomainDetailCommon {
        &mut self.common
    }
}

impl PublicInternetRoutingDomainDetail {
    pub fn new(registry: VeilidComponentRegistry) -> Self {
        Self {
            registry,
            interface_addresses: Default::default(),
            common: RoutingDomainDetailCommon::new(RoutingDomain::PublicInternet),
            published_peer_info: Default::default(),
        }
    }

    #[expect(dead_code)]
    pub fn interface_addresses(&self) -> Vec<IfAddr> {
        self.interface_addresses.clone()
    }

    pub fn set_interface_addresses(&mut self, mut interface_addresses: Vec<IfAddr>) -> bool {
        // Filter out any networks that are only locally routable as the routing domains should not overlap
        interface_addresses.retain(|x| Address::from_ip_addr(x.ip()).is_global());
        interface_addresses.sort();
        if interface_addresses == self.interface_addresses {
            return false;
        }
        self.interface_addresses = interface_addresses;
        true
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", target = "rtab", skip(self), fields(__VEILID_LOG_KEY = self.log_key()), ret))]
    fn get_direct_contact_method(
        &self,
        ctx: &ContactMethodContext,
        target_did: &DialInfoDetail,
    ) -> Option<ContactMethod> {
        // Do we need to signal before going inbound?
        if !target_did.class.requires_signal() {
            // Go direct without signaling
            return Some(ContactMethod::Direct(target_did.dial_info.clone()));
        }

        // Get the target's inbound relay, it must have one or it is not reachable
        for node_b_relay in ctx.peer_b.node_info().relay_info_list() {
            // Note that relay_peer_info could be node_a, in which case a connection already exists
            // and we only get here if the connection had dropped, in which case node_a is unreachable until
            // it gets a new relay connection up
            if node_b_relay
                .node_ids()
                .contains_any_from_slice(ctx.peer_a.node_ids())
            {
                return Some(ContactMethod::Existing);
            }

            // Get best node id to contact relay with
            if !node_b_relay.node_ids().contains_kind(ctx.best_ck) {
                // No best relay id
                return Some(ContactMethod::Unreachable);
            };

            // Can node A reach the inbound relay directly?
            if let Some(node_b_relay_did) = self.best_dial_info_detail_between_nodes(
                ctx.node_a,
                node_b_relay,
                ctx.dial_info_filter,
                ctx.sequencing,
                ctx.context_sort,
            ) {
                // Can node A receive anything inbound ever?
                if ctx.node_a.has_dial_info() {
                    ///////// Reverse connection

                    // Get the best match dial info for an reverse inbound connection from node B to node A
                    if let Some(reverse_did) = self.best_dial_info_detail_between_nodes(
                        ctx.node_b,
                        ctx.node_a,
                        ctx.dial_info_filter,
                        ctx.sequencing,
                        ctx.context_sort,
                    ) {
                        // Ensure we aren't on the same public IP address (no hairpin nat)
                        if reverse_did.dial_info.ip_addr() != target_did.dial_info.ip_addr() {
                            // Can we receive a direct reverse connection?
                            if !reverse_did.class.requires_signal() {
                                return Some(ContactMethod::SignalReverse(
                                    node_b_relay_did.dial_info,
                                ));
                            }
                        }
                    }

                    ///////// UDP hole-punch

                    // Does node B have a direct udp dialinfo node A can reach?
                    let udp_dial_info_filter = ctx
                        .dial_info_filter
                        .filtered(DialInfoFilter::all().with_protocol_type(ProtocolType::UDP));
                    if let Some(target_udp_did) = self.best_dial_info_detail_between_nodes(
                        ctx.node_a,
                        ctx.node_b,
                        udp_dial_info_filter,
                        ctx.sequencing,
                        ctx.context_sort,
                    ) {
                        // Does node A have a direct udp dialinfo that node B can reach?
                        if let Some(reverse_udp_did) = self.best_dial_info_detail_between_nodes(
                            ctx.node_b,
                            ctx.node_a,
                            udp_dial_info_filter,
                            ctx.sequencing,
                            ctx.context_sort,
                        ) {
                            // Ensure we aren't on the same public IP address (no hairpin nat)
                            if reverse_udp_did.dial_info.ip_addr()
                                != target_udp_did.dial_info.ip_addr()
                            {
                                // The target and ourselves have a udp dialinfo that they can reach
                                return Some(ContactMethod::SignalHolePunch(
                                    node_b_relay_did.dial_info,
                                ));
                            }
                        }
                    }
                    // Otherwise we have to inbound relay
                }

                return Some(ContactMethod::InboundRelay(node_b_relay_did.dial_info));
            }
        }
        None
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", target = "rtab", skip(self), fields(__VEILID_LOG_KEY = self.log_key()), ret))]
    fn get_indirect_contact_method(
        &self,
        ctx: &ContactMethodContext,
        node_b_relay: &RelayInfo,
    ) -> Option<ContactMethod> {
        // Note that node_b_relay could be node_a, in which case a connection already exists
        // and we only get here if the connection had dropped, in which case node_b is unreachable until
        // it gets a new relay connection up
        if node_b_relay
            .node_ids()
            .contains_any_from_slice(ctx.peer_a.node_ids())
        {
            return Some(ContactMethod::Existing);
        }

        // Get best node id to contact relay with
        if !node_b_relay.node_ids().contains_kind(ctx.best_ck) {
            // No best relay id
            return Some(ContactMethod::Unreachable);
        };

        // Can we reach the inbound relay?
        if let Some(node_b_relay_did) = self.best_dial_info_detail_between_nodes(
            ctx.node_a,
            node_b_relay,
            ctx.dial_info_filter,
            ctx.sequencing,
            ctx.context_sort,
        ) {
            ///////// Reverse connection

            // Get the best match dial info for an reverse inbound connection from node B to node A
            // unless both nodes are on the same ipblock
            if let Some(reverse_did) = (!ctx.same_ipblock)
                .then(|| {
                    self.best_dial_info_detail_between_nodes(
                        ctx.node_b,
                        ctx.node_a,
                        ctx.dial_info_filter,
                        ctx.sequencing,
                        ctx.context_sort,
                    )
                })
                .flatten()
            {
                // Can we receive a direct reverse connection?
                if !reverse_did.class.requires_signal() {
                    return Some(ContactMethod::SignalReverse(node_b_relay_did.dial_info));
                }
            }

            return Some(ContactMethod::InboundRelay(node_b_relay_did.dial_info));
        }

        None
    }
}

impl RoutingDomainDetail for PublicInternetRoutingDomainDetail {
    fn routing_domain(&self) -> RoutingDomain {
        RoutingDomain::PublicInternet
    }

    fn state(&self) -> RoutingDomainState {
        if !self.common.confirmed() {
            if self.common.address_types().is_empty() || self.common.outbound_protocols().is_empty()
            {
                return RoutingDomainState::Invalid;
            }

            return RoutingDomainState::NeedsDialInfoConfirmation;
        }
        if self.common.address_types().is_empty() || self.common.outbound_protocols().is_empty() {
            return RoutingDomainState::Unusable;
        }

        let relay_status = self.relay_status();
        let needs_relays = relay_status.needs_more_relays();

        // If relays are wanted, they are all allocated at the same time, so we either have all the relays
        // we need, or we don't have any at all
        if needs_relays && self.relays().is_empty() {
            return RoutingDomainState::NeedsRelays { relay_status };
        }

        RoutingDomainState::ReadyToPublish { relay_status }
    }

    fn relay_status(&self) -> RelayStatus {
        RelayStatus::new_from_routing_domain_detail(self)
    }

    fn outbound_protocols(&self) -> ProtocolTypeSet {
        self.common.outbound_protocols()
    }
    fn inbound_protocols(&self) -> ProtocolTypeSet {
        self.common.inbound_protocols()
    }
    fn address_types(&self) -> AddressTypeSet {
        self.common.address_types()
    }
    fn origin_routing_domains(&self) -> RoutingDomainSet {
        RoutingDomain::LocalNetwork | RoutingDomain::PublicInternet
    }
    fn confirmed(&self) -> bool {
        self.common.confirmed()
    }
    fn capabilities(&self) -> Vec<VeilidCapability> {
        self.common.capabilities()
    }
    fn relays(&self) -> Vec<RoutingDomainRelay> {
        self.common.relays()
    }
    fn relays_and_states(&self) -> Vec<(RoutingDomainRelay, RoutingDomainRelayState)> {
        self.common.relays_and_states()
    }

    fn dial_info_details(&self) -> &Vec<DialInfoDetail> {
        self.common.dial_info_details()
    }

    fn is_network_translated(&self) -> bool {
        let mut inbound_addresses = HashSet::<_>::new();
        for did in self.dial_info_details() {
            inbound_addresses.insert(did.dial_info.ip_addr());
        }
        for intf_addr in &self.interface_addresses {
            inbound_addresses.remove(&intf_addr.ip());
        }
        !inbound_addresses.is_empty()
    }

    fn inbound_dial_info_filter(&self) -> DialInfoFilter {
        self.common.inbound_dial_info_filter()
    }
    fn outbound_dial_info_filter(&self) -> DialInfoFilter {
        self.common.outbound_dial_info_filter()
    }

    fn get_peer_info(&self, rti: &RoutingTableInner) -> Arc<PeerInfo> {
        self.common.get_current_peer_info(rti)
    }
    fn get_published_peer_info(&self) -> Option<Arc<PeerInfo>> {
        (*self.published_peer_info.lock()).clone()
    }

    fn get_bootstrap_peers(&self) -> Vec<NodeRef> {
        self.common.get_bootstrap_peers()
    }
    fn clear_bootstrap_peers(&self) {
        self.common.clear_bootstrap_peers();
    }
    fn add_bootstrap_peer(&self, bootstrap_peer: NodeRef) {
        self.common.add_bootstrap_peer(bootstrap_peer)
    }

    ////////////////////////////////////////////////

    fn can_contain_address(&self, address: Address) -> bool {
        address.is_global()
    }

    fn refresh(&self) {
        self.common.clear_current_peer_info_cache();
    }

    fn publish_peer_info(&self, rti: &RoutingTableInner) -> bool {
        let (opt_old_peer_info, opt_new_peer_info) = {
            let opt_new_peer_info = {
                if !matches!(
                    self.state(),
                    RoutingDomainState::ReadyToPublish { relay_status: _ }
                ) {
                    None
                } else {
                    let pi = self.get_peer_info(rti);
                    Some(pi)
                }
            };

            // Don't publish if the peer info hasnt changed from our previous publication
            let mut ppi_lock = self.published_peer_info.lock();
            let opt_old_peer_info = (*ppi_lock).clone();

            if let Some(old_peer_info) = &opt_old_peer_info {
                if let Some(new_peer_info) = &opt_new_peer_info {
                    if new_peer_info.equivalent(old_peer_info) {
                        veilid_log!(rti debug "[PublicInternet] Not publishing peer info because it is equivalent");
                        return false;
                    }
                }
            } else if opt_new_peer_info.is_none() {
                veilid_log!(rti debug "[PublicInternet] Not publishing peer info because it is still None");
                return false;
            }

            if let Some(new_peer_info) = &opt_new_peer_info {
                veilid_log!(rti debug "[PublicInternet] Published new peer info: {}", new_peer_info);
            } else {
                veilid_log!(rti debug "[PublicInternet] Unpublishing because current peer info is invalid");
            }

            *ppi_lock = opt_new_peer_info.clone();

            (opt_old_peer_info, opt_new_peer_info)
        };

        if let Err(e) = rti.event_bus().post(PeerInfoChangeEvent {
            routing_domain: RoutingDomain::PublicInternet,
            opt_old_peer_info,
            opt_new_peer_info,
        }) {
            veilid_log!(rti debug "Failed to post event: {}", e);
        }

        true
    }

    fn unpublish_peer_info(&self, rti: &RoutingTableInner) {
        let mut ppi_lock = self.published_peer_info.lock();
        veilid_log!(rti debug "[PublicInternet] Unpublished peer info");
        let opt_old_peer_info = ppi_lock.clone();
        *ppi_lock = None;

        if let Err(e) = rti.event_bus().post(PeerInfoChangeEvent {
            routing_domain: RoutingDomain::PublicInternet,
            opt_old_peer_info,
            opt_new_peer_info: None,
        }) {
            veilid_log!(rti debug "Failed to post event: {}", e);
        }
    }

    fn ensure_dial_info_is_valid(&self, dial_info: &DialInfo) -> bool {
        let address = dial_info.socket_address().address();
        let can_contain_address = self.can_contain_address(address);

        if !can_contain_address {
            return false;
        }
        if !dial_info.is_valid() {
            veilid_log!(self debug
                "shouldn't be registering invalid addresses: {:?}",
                dial_info
            );
            return false;
        }
        true
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", target = "rtab", skip(self, rti, context_sort), fields(__VEILID_LOG_KEY = self.log_key()), ret))]
    fn get_contact_method(
        &self,
        rti: &RoutingTableInner,
        peer_a: Arc<PeerInfo>,
        peer_b: Arc<PeerInfo>,
        dial_info_filter: DialInfoFilter,
        sequencing: Sequencing,
        context_sort: Option<&DialInfoDetailSort>,
    ) -> ContactMethod {
        let ip6_prefix_size = rti.config().network.max_connections_per_ip6_prefix_size as usize;

        // Get the nodeinfos for convenience
        let node_a = peer_a.node_info();
        let node_b = peer_b.node_info();

        // Check to see if these nodes are on the same network
        let same_ipblock = node_a.is_on_same_ipblock(node_b, ip6_prefix_size);

        // Get the node ids that would be used between these peers
        let cck = common_crypto_kinds(&peer_a.node_ids().kinds(), &peer_b.node_ids().kinds());
        let Some(best_ck) = cck.first().copied() else {
            // No common crypto kinds between these nodes, can't contact
            return ContactMethod::Unreachable;
        };

        //let node_a_id = peer_a.node_ids().get(best_ck).unwrap_or_log();
        let node_b_id = peer_b.node_ids().get(best_ck).unwrap_or_log();

        let ctx = ContactMethodContext {
            //rti,
            peer_a: peer_a.clone(),
            peer_b: peer_b.clone(),
            node_a,
            node_b,
            dial_info_filter,
            sequencing,
            context_sort,
            best_ck,
            same_ipblock,
            //node_a_id,
            node_b_id,
        };

        // Get the best match dial info for node B if we have it
        // To avoid hairpin NAT, if both nodes are on the same IP block
        // We should try to contact node B's relay(s) first before trying a direct address
        let mut tried_inbound_relaying = false;
        if same_ipblock {
            for node_b_relay in ctx.peer_b.node_info().relay_info_list() {
                if let Some(out) = self.get_indirect_contact_method(&ctx, node_b_relay) {
                    return out;
                }
            }
            tried_inbound_relaying = true;
        }

        // Now let's try to reach the node directly
        if let Some(target_did) = self.best_dial_info_detail_between_nodes(
            node_a,
            node_b,
            dial_info_filter,
            sequencing,
            context_sort,
        ) {
            if let Some(out) = self.get_direct_contact_method(&ctx, &target_did) {
                return out;
            }
        }

        if !tried_inbound_relaying {
            // If the node B can not be reached directly and we didn't try inbound relaying first, give it a shot
            for node_b_relay in ctx.peer_b.node_info().relay_info_list() {
                if let Some(out) = self.get_indirect_contact_method(&ctx, node_b_relay) {
                    return out;
                }
            }
        }

        // If node A can't reach the node by other means, it may need to use its outbound relay
        for node_a_relay in ctx.peer_a.node_info().relay_info_list() {
            if !matches!(node_a_relay.relay_kind(), RelayKind::Outbound) {
                continue;
            }
            if let Some(node_a_relay_id) = node_a_relay.node_ids().get(best_ck) {
                // Ensure it's not our relay we're trying to reach
                if node_a_relay_id != ctx.node_b_id {
                    return ContactMethod::OutboundRelay(node_a_relay_id);
                }
            }
        }

        ContactMethod::Unreachable
    }

    fn debug(&self, alt: bool) -> String {
        if alt {
            format!("{:#?}", self)
        } else {
            format!("{:?}", self)
        }
    }
}
