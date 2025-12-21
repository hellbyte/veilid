mod editor;
mod local_network;
mod public_internet;
mod relay_status;
mod routing_domain_relay;
mod routing_domain_state;

use super::*;

pub use editor::*;
pub use local_network::*;
pub use public_internet::*;
pub use relay_status::*;
pub use routing_domain_relay::*;
pub use routing_domain_state::*;

/// General trait for all routing domains
pub trait RoutingDomainDetail: VeilidComponentRegistryAccessor {
    // Common accessors
    fn routing_domain(&self) -> RoutingDomain;
    fn state(&self) -> RoutingDomainState;
    fn relay_status(&self) -> RelayStatus;
    fn outbound_protocols(&self) -> ProtocolTypeSet;
    fn inbound_protocols(&self) -> ProtocolTypeSet;
    fn address_types(&self) -> AddressTypeSet;
    fn origin_routing_domains(&self) -> RoutingDomainSet;
    fn confirmed(&self) -> bool;
    fn capabilities(&self) -> Vec<VeilidCapability>;
    fn relays(&self) -> Vec<RoutingDomainRelay>;
    fn relays_and_states(&self) -> Vec<(RoutingDomainRelay, RoutingDomainRelayState)>;
    fn dial_info_details(&self) -> &Vec<DialInfoDetail>;
    fn is_network_translated(&self) -> bool;
    fn get_published_peer_info(&self) -> Option<Arc<PeerInfo>>;
    fn inbound_dial_info_filter(&self) -> DialInfoFilter;
    fn outbound_dial_info_filter(&self) -> DialInfoFilter;
    fn get_peer_info(&self, rti: &RoutingTableInner) -> Arc<PeerInfo>;

    // Can this routing domain contain a particular address
    fn can_contain_address(&self, address: Address) -> bool;
    fn ensure_dial_info_is_valid(&self, dial_info: &DialInfo) -> bool;

    // Refresh caches if external data changes
    fn refresh(&self);

    // Publish current peer info to the world
    fn publish_peer_info(&self, rti: &RoutingTableInner) -> bool;
    fn unpublish_peer_info(&self, rti: &RoutingTableInner);

    // Get the contact method required for node A to reach node B in this routing domain
    // Routing table must be locked for reading to use this function
    fn get_contact_method(
        &self,
        rti: &RoutingTableInner,
        peer_a: Arc<PeerInfo>,
        peer_b: Arc<PeerInfo>,
        dial_info_filter: DialInfoFilter,
        sequencing: Sequencing,
        context_sort: Option<&DialInfoDetailSort>,
    ) -> ContactMethod;

    /// Gets the best dial info to attempt a connection between two nodes
    /// The best available ordering mode is selected from the sequencing constraint,
    /// then the dial info filter is applied, and the available
    /// dial info are then sorted by the context_sort. The first item in the list
    /// is then returned.
    #[instrument(level = "trace", target = "rtab", skip(self, context_sort), fields(__VEILID_LOG_KEY = self.log_key()), ret)]
    fn best_dial_info_detail_between_nodes(
        &self,
        from_node: &NodeInfo,
        to_node: &dyn HasDialInfoDetailList,
        dial_info_filter: DialInfoFilter,
        sequencing: Sequencing,
        context_sort: Option<&DialInfoDetailSort>,
    ) -> Option<DialInfoDetail> {
        // Consider outbound capabilities
        let (ordering, outbound_dial_info_filter) = DialInfoFilter::all()
            .with_address_type_set(from_node.address_types())
            .with_protocol_type_set(from_node.outbound_protocols())
            .apply_sequencing(sequencing);

        // If the filter is dead then we won't be able to connect
        if dial_info_filter.is_dead() {
            return None;
        }

        // Get the sort for this ordering mode
        let ordering_sort: Option<Box<DialInfoDetailSort>> =
            DialInfoDetail::get_ordering_sort(ordering);

        // Get all dial info we could possibly connect to for node B with the selected sequencing
        let filter =
            Box::new(|did: &DialInfoDetail| did.matches_filter(&outbound_dial_info_filter));
        let mut all_reachable_dial_info =
            to_node.filtered_dial_info_details(ordering_sort.as_deref(), &filter);

        // Get the best available ordering mode for the reachable dial info
        let best_sequence_ordering = all_reachable_dial_info
            .iter()
            .map(|x| x.dial_info.protocol_type().sequence_ordering())
            .reduce(|a, b| a.max(b))?;

        // Retain only the dial info with the best sequence ordering and matching the dial info filter
        all_reachable_dial_info.retain(|x| {
            (matches!(sequencing, Sequencing::NoPreference)
                || x.dial_info.protocol_type().sequence_ordering() == best_sequence_ordering)
                && x.matches_filter(&dial_info_filter)
        });

        // Of the remaining candidates, apply the context sort to allow for reordering based on
        // recent dial info failures and any other environmental conditions
        if let Some(context_sort) = context_sort {
            all_reachable_dial_info.sort_by(context_sort);
        }

        // Now return the first dial info detail in the list
        all_reachable_dial_info.into_iter().next()
    }

    // Bootstrap peers
    fn get_bootstrap_peers(&self) -> Vec<NodeRef>;
    fn clear_bootstrap_peers(&self);
    fn add_bootstrap_peer(&self, bootstrap_peer: NodeRef);

    // Debugging
    fn debug(&self, alt: bool) -> String;
}

trait RoutingDomainDetailCommonAccessors: RoutingDomainDetail {
    #[expect(dead_code)]
    fn common(&self) -> &RoutingDomainDetailCommon;
    fn common_mut(&mut self) -> &mut RoutingDomainDetailCommon;
}

#[derive(Debug)]
struct RoutingDomainDetailCommon {
    routing_domain: RoutingDomain,
    outbound_protocols: ProtocolTypeSet,
    inbound_protocols: ProtocolTypeSet,
    address_types: AddressTypeSet,
    relays_and_states: Vec<(RoutingDomainRelay, RoutingDomainRelayState)>,
    capabilities: Vec<VeilidCapability>,
    dial_info_details: Vec<DialInfoDetail>,
    confirmed: bool,
    // caches
    current_peer_info_cache: Mutex<Option<Arc<PeerInfo>>>,
    bootstrap_peers: Mutex<Vec<NodeRef>>,
}

impl RoutingDomainDetailCommon {
    pub fn new(routing_domain: RoutingDomain) -> Self {
        Self {
            routing_domain,
            outbound_protocols: Default::default(),
            inbound_protocols: Default::default(),
            address_types: Default::default(),
            relays_and_states: Default::default(),
            capabilities: Default::default(),
            dial_info_details: Default::default(),
            confirmed: false,
            current_peer_info_cache: Mutex::new(Default::default()),
            bootstrap_peers: Mutex::new(Default::default()),
        }
    }

    ///////////////////////////////////////////////////////////////////////
    // Accessors

    pub fn confirmed(&self) -> bool {
        self.confirmed
    }

    pub fn outbound_protocols(&self) -> ProtocolTypeSet {
        self.outbound_protocols
    }

    pub fn inbound_protocols(&self) -> ProtocolTypeSet {
        self.inbound_protocols
    }

    pub fn address_types(&self) -> AddressTypeSet {
        self.address_types
    }

    pub fn capabilities(&self) -> Vec<VeilidCapability> {
        self.capabilities.clone()
    }

    pub fn get_bootstrap_peers(&self) -> Vec<NodeRef> {
        self.bootstrap_peers.lock().clone()
    }

    pub fn clear_bootstrap_peers(&self) {
        self.bootstrap_peers.lock().clear()
    }

    pub fn add_bootstrap_peer(&self, bootstrap_peer: NodeRef) {
        let mut bootstrap_peers = self.bootstrap_peers.lock();
        bootstrap_peers.push(bootstrap_peer);
    }

    pub fn relays(&self) -> Vec<RoutingDomainRelay> {
        self.relays_and_states.iter().map(|x| x.0.clone()).collect()
    }

    pub fn relays_and_states(&self) -> Vec<(RoutingDomainRelay, RoutingDomainRelayState)> {
        self.relays_and_states.clone()
    }

    pub fn dial_info_details(&self) -> &Vec<DialInfoDetail> {
        &self.dial_info_details
    }

    pub fn inbound_dial_info_filter(&self) -> DialInfoFilter {
        DialInfoFilter::all()
            .with_protocol_type_set(self.inbound_protocols)
            .with_address_type_set(self.address_types)
    }

    pub fn outbound_dial_info_filter(&self) -> DialInfoFilter {
        DialInfoFilter::all()
            .with_protocol_type_set(self.outbound_protocols)
            .with_address_type_set(self.address_types)
    }

    pub fn get_current_peer_info(&self, rti: &RoutingTableInner) -> Arc<PeerInfo> {
        let mut cpi = self.current_peer_info_cache.lock();
        if cpi.is_none() {
            // Regenerate peer info
            let pi = self.make_current_peer_info(rti);

            // Cache the peer info
            *cpi = Some(Arc::new(pi));
        }
        cpi.as_ref().unwrap().clone()
    }

    ///////////////////////////////////////////////////////////////////////
    // Mutators

    fn setup_network(
        &mut self,
        outbound_protocols: ProtocolTypeSet,
        inbound_protocols: ProtocolTypeSet,
        address_types: AddressTypeSet,
        capabilities: Vec<VeilidCapability>,
        confirmed: bool,
    ) {
        self.outbound_protocols = outbound_protocols;
        self.inbound_protocols = inbound_protocols;
        self.address_types = address_types;
        self.capabilities = capabilities;
        self.confirmed = confirmed;
        self.clear_current_peer_info_cache();
    }

    fn clear_dial_info_details(
        &mut self,
        address_type: Option<AddressType>,
        protocol_type: Option<ProtocolType>,
    ) {
        self.dial_info_details.retain_mut(|e| {
            let mut remove = true;
            if let Some(pt) = protocol_type {
                if pt != e.dial_info.protocol_type() {
                    remove = false;
                }
            }
            if let Some(at) = address_type {
                if at != e.dial_info.address_type() {
                    remove = false;
                }
            }
            !remove
        });
        self.clear_current_peer_info_cache();
    }
    fn add_dial_info_detail(&mut self, did: DialInfoDetail) {
        self.dial_info_details.push(did);
        self.dial_info_details.sort();
        self.dial_info_details.dedup();
        self.clear_current_peer_info_cache();
    }
    // fn remove_dial_info_detail(&mut self, did: DialInfoDetail) {
    //     if let Some(index) = self.dial_info_details.iter().position(|x| *x == did) {
    //         self.dial_info_details.remove(index);
    //     }
    //     self.clear_cache();
    // }

    fn set_relays(&mut self, relays: Vec<RoutingDomainRelay>) {
        let changed = self.relays() != relays;
        if !changed {
            return;
        }

        let cur_ts = Timestamp::now();

        let mut new_relays_and_states = vec![];
        for relay in relays {
            // See if this relay exists already, if so, keep its state
            if let Some(existing_relay_state) =
                self.relays_and_states.iter().find_map(
                    |(r, s)| {
                        if r == &relay {
                            Some(*s)
                        } else {
                            None
                        }
                    },
                )
            {
                new_relays_and_states.push((relay, existing_relay_state));
            } else {
                // New relay, make a new state for it
                new_relays_and_states.push((
                    relay,
                    RoutingDomainRelayState {
                        last_keepalive: cur_ts,
                        last_optimized: cur_ts,
                    },
                ));
            }
        }
        self.relays_and_states = new_relays_and_states;
        self.clear_current_peer_info_cache();
    }

    fn set_relay_state(&mut self, relay: &RoutingDomainRelay, state: RoutingDomainRelayState) {
        let Some(existing_relay_state) = self.relays_and_states.iter_mut().find_map(|x| {
            if &x.0 == relay {
                Some(&mut x.1)
            } else {
                None
            }
        }) else {
            return;
        };
        *existing_relay_state = state;
    }

    //////////////////////////////////////////////////////////////////////////////
    // Internal functions

    fn make_current_peer_info(&self, rti: &RoutingTableInner) -> PeerInfo {
        let routing_table = rti.routing_table();
        let cur_ts = Timestamp::now_non_decreasing();
        let mut relay_info_list = vec![];
        for relay in self.relays() {
            let relay_node_ids = relay.relay_node.locked(rti).node_ids();
            let Some(relay_node_info) = relay.relay_node.locked(rti).node_info(self.routing_domain)
            else {
                veilid_log!(rti debug "not including relay node {} in peer info for routing domain {:?}", relay_node_ids, self.routing_domain);
                continue;
            };
            let relay_info = RelayInfo::new(
                relay_node_info.timestamp(),
                relay_node_ids,
                relay_node_info.outbound_protocols(),
                relay_node_info.address_types(),
                relay.dial_info_details,
                relay.relay_kind,
            );
            relay_info_list.push(relay_info);
        }

        let keypairs = routing_table.signing_key_pairs();
        let public_keys: Vec<_> = keypairs.iter().map(|x| x.key()).collect();
        let secret_keys =
            SecretKeyGroup::from(keypairs.iter().map(|x| x.secret()).collect::<Vec<_>>());
        let crypto_info_list: Vec<_> = public_keys
            .iter()
            .map(|pk| match pk.kind() {
                CRYPTO_KIND_VLD0 => CryptoInfo::VLD0 {
                    public_key: pk.value(),
                },
                _ => {
                    unimplemented!("Must implement cryptoinfo")
                }
            })
            .collect();

        let node_info = NodeInfo::new(
            cur_ts,
            VALID_ENVELOPE_VERSIONS.to_vec(),
            crypto_info_list,
            self.capabilities.clone(),
            self.outbound_protocols,
            self.address_types,
            self.dial_info_details.clone(),
            relay_info_list,
        );

        PeerInfo::new_from_node_info(&routing_table, self.routing_domain, &secret_keys, node_info)
            .expect("our own peerinfo should never fail")
    }

    fn clear_current_peer_info_cache(&self) {
        *self.current_peer_info_cache.lock() = None;
    }
}
