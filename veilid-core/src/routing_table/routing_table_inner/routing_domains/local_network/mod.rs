mod editor;

pub use editor::*;

use super::*;

impl_veilid_log_facility!("rtab");

/// Local Network routing domain internals
pub struct LocalNetworkRoutingDomainDetail {
    /// Registry accessor
    registry: VeilidComponentRegistry,
    /// The interface networks that are in this domain
    interface_addresses: Vec<IfAddr>,
    /// Common implementation for all routing domains
    common: RoutingDomainDetailCommon,
    /// Published peer info for this routing domain
    published_peer_info: Mutex<Option<Arc<PeerInfo>>>,
}

impl fmt::Debug for LocalNetworkRoutingDomainDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalNetworkRoutingDomainDetail")
            // .field("registry", &self.registry)
            .field("interface_addresses", &self.interface_addresses)
            .field("common", &self.common)
            .field("published_peer_info", &self.published_peer_info)
            .finish()
    }
}

impl_veilid_component_accessors!(LocalNetworkRoutingDomainDetail);

impl LocalNetworkRoutingDomainDetail {
    pub fn new(registry: VeilidComponentRegistry) -> Self {
        Self {
            registry,
            interface_addresses: Default::default(),
            common: RoutingDomainDetailCommon::new(RoutingDomain::LocalNetwork),
            published_peer_info: Default::default(),
        }
    }
}

impl LocalNetworkRoutingDomainDetail {
    #[expect(dead_code)]
    pub fn interface_addresses(&self) -> Vec<IfAddr> {
        self.interface_addresses.clone()
    }

    pub fn set_interface_addresses(&mut self, mut interface_addresses: Vec<IfAddr>) -> bool {
        // Filter out any networks that are publicly routable as the routing domains should not overlap
        interface_addresses.retain(|x| Address::from_ip_addr(x.ip()).is_local());
        interface_addresses.sort();
        if interface_addresses == self.interface_addresses {
            return false;
        }
        self.interface_addresses = interface_addresses;
        true
    }
}

impl RoutingDomainDetailCommonAccessors for LocalNetworkRoutingDomainDetail {
    fn common(&self) -> &RoutingDomainDetailCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut RoutingDomainDetailCommon {
        &mut self.common
    }
}

impl RoutingDomainDetail for LocalNetworkRoutingDomainDetail {
    fn routing_domain(&self) -> RoutingDomain {
        RoutingDomain::LocalNetwork
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
        // No relay support for LocalNetwork domain yet
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
        RoutingDomain::LocalNetwork.into()
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

    fn can_contain_address(&self, address: Address) -> bool {
        if address.is_global() {
            return false;
        }

        let ip = address.ip_addr();
        for localnet in &self.interface_addresses {
            if ipaddr_in_network(ip, localnet.network().ip(), localnet.netmask()) {
                return true;
            }
        }
        false
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
                        veilid_log!(rti debug "[LocalNetwork] Not publishing peer info because it is equivalent");
                        return false;
                    }
                }
            } else if opt_new_peer_info.is_none() {
                veilid_log!(rti debug "[LocalNetwork] Not publishing peer info because it is still None");
                return false;
            }

            if let Some(new_peer_info) = &opt_new_peer_info {
                veilid_log!(rti debug "[LocalNetwork] Published new peer info: {}", new_peer_info);
            } else {
                veilid_log!(rti debug "[LocalNetwork] Unpublishing because current peer info is invalid");
            }
            *ppi_lock = opt_new_peer_info.clone();

            (opt_old_peer_info, opt_new_peer_info)
        };

        if let Err(e) = rti.event_bus().post(PeerInfoChangeEvent {
            routing_domain: RoutingDomain::LocalNetwork,
            opt_old_peer_info,
            opt_new_peer_info,
        }) {
            veilid_log!(rti debug "Failed to post event: {}", e);
        }

        true
    }

    fn unpublish_peer_info(&self, rti: &RoutingTableInner) {
        let mut ppi_lock = self.published_peer_info.lock();
        veilid_log!(rti debug "[LocalNetwork] Unpublished peer info");
        let opt_old_peer_info = ppi_lock.clone();
        *ppi_lock = None;

        if let Err(e) = rti.event_bus().post(PeerInfoChangeEvent {
            routing_domain: RoutingDomain::LocalNetwork,
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

    #[cfg_attr(feature = "instrument", instrument(level = "trace", target = "rtab", skip(self, _rti, context_sort), fields(__VEILID_LOG_KEY = self.log_key()), ret))]
    fn get_contact_method(
        &self,
        _rti: &RoutingTableInner,
        peer_a: Arc<PeerInfo>,
        peer_b: Arc<PeerInfo>,
        dial_info_filter: DialInfoFilter,
        sequencing: Sequencing,
        context_sort: Option<&DialInfoDetailSort>,
    ) -> ContactMethod {
        // Get the nodeinfos for convenience
        let node_a = peer_a.node_info();
        let node_b = peer_b.node_info();

        // Get the node ids that would be used between these peers
        let cck = common_crypto_kinds(&peer_a.node_ids().kinds(), &peer_b.node_ids().kinds());
        let Some(_best_ck) = cck.first().copied() else {
            // No common crypto kinds between these nodes, can't contact
            return ContactMethod::Unreachable;
        };

        if let Some(target_did) = self.best_dial_info_detail_between_nodes(
            node_a,
            node_b,
            dial_info_filter,
            sequencing,
            context_sort,
        ) {
            match target_did.class {
                DialInfoClass::Direct => return ContactMethod::Direct(target_did.dial_info),
                DialInfoClass::Mapped
                | DialInfoClass::FullConeNAT
                | DialInfoClass::Blocked
                | DialInfoClass::AddressRestrictedNAT
                | DialInfoClass::PortRestrictedNAT => {
                    veilid_log!(self warn "LocalNetwork dial info found with non-direct class: {}:\n{:#?}", target_did, peer_b);
                    return ContactMethod::Unreachable;
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
