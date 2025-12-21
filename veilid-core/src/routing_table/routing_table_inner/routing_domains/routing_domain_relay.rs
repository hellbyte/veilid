use super::*;

#[derive(Debug, Clone)]
pub struct RoutingDomainRelay {
    pub relay_node: FilteredNodeRef,
    pub relay_kind: RelayKind,
    pub pings: Vec<RelayPing>,
    pub dial_info_details: Vec<DialInfoDetail>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RoutingDomainRelayState {
    pub last_keepalive: Timestamp,
    pub last_optimized: Timestamp,
}

impl PartialEq for RoutingDomainRelay {
    fn eq(&self, other: &Self) -> bool {
        self.relay_node.equivalent(&other.relay_node)
            && self.relay_kind == other.relay_kind
            && self.pings == other.pings
            && self.dial_info_details == other.dial_info_details
    }
}

impl Eq for RoutingDomainRelay {}

impl RoutingDomainRelay {
    pub fn new(routing_domain: RoutingDomain, relay_node: NodeRef, relay_kind: RelayKind) -> Self {
        RoutingDomainRelay {
            relay_node: relay_node
                .custom_filtered(NodeRefFilter::new().with_routing_domain(routing_domain)),
            relay_kind,
            dial_info_details: vec![],
            pings: vec![],
        }
    }
}
