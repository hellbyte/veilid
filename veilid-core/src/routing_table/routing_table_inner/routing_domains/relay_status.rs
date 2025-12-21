use super::*;

#[derive(Debug, Clone)]
pub struct RelayPing {
    pub node_ref: FilteredNodeRef,
}

impl PartialEq for RelayPing {
    fn eq(&self, other: &Self) -> bool {
        self.node_ref.equivalent(&other.node_ref)
    }
}

impl Eq for RelayPing {}

/// The current node's relaying capabilities and requirements
#[derive(Debug, Clone)]
pub struct RelayStatus {
    /// Routing domain this is for
    pub routing_domain: RoutingDomain,
    /// Low level port info for this node
    /// This is which ports are mapped externally that we may need keepalive pings for
    pub low_level_port_info: LowLevelPortInfo,
    /// This node's outbound dial info filter
    /// Used to determine if a relay's dialinfo is directly reachable
    pub dial_info_filter: DialInfoFilter,
    /// All protocol/address types requiring inbound relays for this node
    pub want_relay_protocols: HashSet<(ProtocolType, AddressType)>,
    /// Ordering modes we still need for relaying, per address type
    pub want_relay_orderings: HashSet<(SequenceOrdering, AddressType)>,
    /// All protocol/address types we can offer inbound relaying for from this node
    #[expect(dead_code)]
    pub can_relay_protocols: HashSet<(ProtocolType, AddressType)>,
    /// All the low level protocols and ports that require nat keepalive pings
    pub wants_nat_keepalives: LowLevelProtocolPorts,
    /// All of the relays and their configuration currently included in our status
    pub relays: Vec<RoutingDomainRelay>,
}

impl RelayStatus {
    pub fn new_from_routing_domain_detail(rdd: &dyn RoutingDomainDetail) -> Self {
        // Make temporary nodeinfo without relay info or keys
        let node_info = NodeInfo::new(
            Timestamp::now_non_decreasing(),
            VALID_ENVELOPE_VERSIONS.to_vec(),
            vec![],
            rdd.capabilities(),
            rdd.outbound_protocols(),
            rdd.address_types(),
            rdd.dial_info_details().clone(),
            vec![],
        );

        Self::new_from_node_info(
            rdd.routing_domain(),
            &node_info,
            rdd.is_network_translated(),
        )
    }

    fn new_from_node_info(
        routing_domain: RoutingDomain,
        node_info: &NodeInfo,
        needs_hairpin_nat_support: bool,
    ) -> Self {
        let low_level_port_info = node_info.get_low_level_port_info();
        let dial_info_filter = DialInfoFilter::all()
            .with_protocol_type_set(node_info.outbound_protocols())
            .with_address_type_set(node_info.address_types());

        // Determine ordering modes we need relaying for
        let ordering_modes = node_info
            .outbound_protocols()
            .iter()
            .map(|x| x.sequence_ordering())
            .collect::<HashSet<_>>();

        // Get the dial info list in preferred deterministic order
        let mut dial_info_list = node_info.dial_info_detail_list().to_vec();
        dial_info_list.sort_by(DialInfoDetail::ordered_sequencing_sort);

        // Figure out which dial info combinations we have that are direct-capable
        let mut direct_did_protoaddrs = HashSet::<(ProtocolType, AddressType)>::new();
        let mut wants_nat_keepalives = LowLevelProtocolPorts::new();
        for did in dial_info_list {
            if !did.class.requires_signal() {
                direct_did_protoaddrs
                    .insert((did.dial_info.protocol_type(), did.dial_info.address_type()));
            }
            if did.class.wants_nat_keepalive() {
                wants_nat_keepalives.insert((
                    did.dial_info.protocol_type().low_level_protocol_type(),
                    did.dial_info.address_type(),
                    did.dial_info.port(),
                ));
            }
        }

        // Calculate which address/protocol type combinations we require
        // relays for, and which we can offer relay support for
        // Get the ordering modes per address type we need, at a minimum, to be able to publish peer info
        let mut want_relay_protocols = HashSet::<(ProtocolType, AddressType)>::new();
        let mut want_relay_orderings = AddressTypeSet::all()
            .iter()
            .flat_map(|at| ordering_modes.iter().map(move |om| (*om, at)))
            .collect::<HashSet<_>>();
        let mut can_relay_protocols = HashSet::<(ProtocolType, AddressType)>::new();

        for at in AddressTypeSet::all() {
            for pt in ProtocolTypeSet::all() {
                // if we can't use this protocol because we don't have its ordering mode enabled at all, then we should exclude it
                if !ordering_modes.contains(&pt.sequence_ordering()) {
                    continue;
                }

                if direct_did_protoaddrs.contains(&(pt, at)) {
                    // We can relay this combination
                    can_relay_protocols.insert((pt, at));

                    // Note the relay ordering
                    want_relay_orderings.remove(&(pt.sequence_ordering(), at));
                }

                if needs_hairpin_nat_support || !direct_did_protoaddrs.contains(&(pt, at)) {
                    // We can't relay this, so we must need a relay for it ourselves
                    // Or we want to allocate a hairpin NAT relay
                    want_relay_protocols.insert((pt, at));
                }
            }
        }

        RelayStatus {
            routing_domain,
            low_level_port_info,
            dial_info_filter,
            want_relay_protocols,
            want_relay_orderings,
            can_relay_protocols,
            wants_nat_keepalives,
            relays: vec![],
        }
    }

    /// Check if we would like more relays
    pub fn wants_more_relays(&self) -> bool {
        // If we want more keepalives for NAT, we want more relays
        if !self.wants_nat_keepalives.is_empty() {
            return true;
        }
        // If there are any protocol/address type combinations we need
        // relaying for still, we want more relays
        !self.want_relay_protocols.is_empty()
    }

    /// Check if we need more relays before publication
    pub fn needs_more_relays(&self) -> bool {
        // If we want more keepalives for NAT, we need more relays
        if !self.wants_nat_keepalives.is_empty() {
            return true;
        }

        // If we have any address type that doesn't need relays for its ordering modes
        // then we don't need more relays (we still want more relays though)
        let mut ready_address_types = AddressTypeSet::all();
        for nro in &self.want_relay_orderings {
            ready_address_types.remove(nro.1);
        }
        ready_address_types.is_empty()
    }

    /// Get the routing domain relays list when we're done
    pub fn get_sorted_relays_list(&self) -> Vec<RoutingDomainRelay> {
        let mut relays = self.relays.clone();

        // Sort things in order of relay preference, using least-capable relays first
        relays.sort_by(|ardr, brdr| {
            // Get address types and protocol types to sort by
            let mut aats = AddressTypeSet::new();
            let mut apts = ProtocolTypeSet::new();
            for did in &ardr.dial_info_details {
                aats |= did.dial_info.address_type();
                apts |= did.dial_info.protocol_type();
            }

            let mut bats = AddressTypeSet::new();
            let mut bpts = ProtocolTypeSet::new();
            for did in &brdr.dial_info_details {
                bats |= did.dial_info.address_type();
                bpts |= did.dial_info.protocol_type();
            }

            // Compare by address type set first (fewer address types is less)
            let c = aats.len().cmp(&bats.len());
            if c != cmp::Ordering::Equal {
                return c;
            }
            for at in AddressTypeSet::all() {
                let a = aats.contains(at);
                let b = bats.contains(at);
                let c = a.cmp(&b);
                if c != cmp::Ordering::Equal {
                    return c;
                }
            }

            // Compare by protocol types set second (fewer protocol types is less)
            let c = apts.len().cmp(&bpts.len());
            if c != cmp::Ordering::Equal {
                return c;
            }
            for pt in ProtocolTypeSet::all() {
                let a = apts.contains(pt);
                let b = bpts.contains(pt);
                let c = a.cmp(&b);
                if c != cmp::Ordering::Equal {
                    return c;
                }
            }

            // Then just compare by node id lists, so things are stable
            let a_nodes = ardr.relay_node.node_ids().to_vec();
            let b_nodes = brdr.relay_node.node_ids().to_vec();

            a_nodes.cmp(&b_nodes)
        });
        relays
    }

    /// Remove a relay's capabilities from our current requirements and determine which
    /// pings should be performed.
    /// Returns true if the requirements changed, or false if applying the relay had no effect
    pub fn apply_relay(&mut self, mut relay: RoutingDomainRelay) -> bool {
        // Make sure this relay is the correct routing domain and has peer info
        let Some(relay_peer_info) = relay.relay_node.get_peer_info(self.routing_domain) else {
            return false;
        };

        // Clear out the dial info details and the pings because we'll add new ones
        relay.dial_info_details.clear();
        relay.pings.clear();

        // For all for the relay's dial info, see if it matches a protocol+address type we need covered
        let mut dial_info_list = relay_peer_info.node_info().dial_info_detail_list().to_vec();
        dial_info_list.sort_by(DialInfoDetail::ordered_sequencing_sort);

        // Determine for this relay, if there are dialinfo that are reachable with our node's
        // dialinfo filter, and which ordering modes can be satisfied by those flows

        let mut possible_ordering_modes = SequenceOrderingSet::new();
        for did in &dial_info_list {
            if did.class.requires_signal() {
                continue;
            }
            let didpa = (did.dial_info.protocol_type(), did.dial_info.address_type());

            // If this dial info can be contacted directly, then it can be used for receiving
            // relaying and satsifying an ordering mode
            if did.dial_info.matches_filter(&self.dial_info_filter) {
                possible_ordering_modes.insert(didpa.0.sequence_ordering());
            }
        }

        // If we did not get a single ordering mode we need for relaying, then this relay is disqualified
        // because we can't connect to it with our outbound protocols/address types directly
        if possible_ordering_modes.is_empty() {
            return false;
        }

        let mut useful = false;

        // Determine relay dial infos we can use from this relay out of our set of needed relay combinations
        // Builds up a set of needed ordering modes to keep flows open for the dial infos we are getting relayed
        for did in &dial_info_list {
            if did.class.requires_signal() {
                continue;
            }
            let didpa = (did.dial_info.protocol_type(), did.dial_info.address_type());

            if self.want_relay_protocols.remove(&didpa) {
                // Still needed this protocol+address type
                useful = true;

                // Mark this dial info as one we're using
                relay.dial_info_details.push(did.clone());

                // Mark this ordering mode as satisfied
                self.want_relay_orderings
                    .remove(&(didpa.0.sequence_ordering(), didpa.1));
            }
        }

        // Collect pings we can use from this relay
        for did in &dial_info_list {
            if did.class.requires_signal() {
                continue;
            }
            let didpa = (did.dial_info.protocol_type(), did.dial_info.address_type());

            // If this dial info can be contacted directly, then it is a ping candidate
            if did.dial_info.matches_filter(&self.dial_info_filter) {
                // See if we should add this ping
                let mut add_ping = false;

                // See if we should add the ping for ordering mode coverage
                let ordering = didpa.0.sequence_ordering();
                add_ping |= possible_ordering_modes.remove(ordering);

                // See if we should add the ping for low level port mapping coverage
                if let Some((llpt, port)) = self
                    .low_level_port_info
                    .protocol_to_port
                    .get(&didpa)
                    .copied()
                {
                    let wnk = (llpt, didpa.1, port);
                    add_ping |= self.wants_nat_keepalives.remove(&wnk);
                }

                // Add the ping if we determined we could use it
                if add_ping {
                    relay.pings.push(RelayPing {
                        node_ref: relay.relay_node.unfiltered().custom_filtered(
                            NodeRefFilter::new()
                                .with_routing_domain(self.routing_domain)
                                .with_dial_info_filter(did.dial_info.make_filter()),
                        ),
                    });
                }
            }
        }

        // Add a relay info to our list if it turned out to be useful
        if useful {
            self.relays.push(relay);
        }

        useful
    }
}
