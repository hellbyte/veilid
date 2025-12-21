use super::*;

/// Just enough information to describe a single relay, for the RLAY capability
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RelayInfo {
    timestamp: Timestamp,
    node_ids: NodeIdGroup,
    outbound_protocols: ProtocolTypeSet,
    address_types: AddressTypeSet,
    dial_info_detail_list: Vec<DialInfoDetail>,
    relay_kind: RelayKind,
}

impl fmt::Display for RelayInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "timestamp: {}", self.timestamp)?;
        writeln!(f, "node_ids: {}", self.node_ids)?;
        writeln!(f, "outbound_protocols: {}", self.outbound_protocols)?;
        writeln!(f, "address_types: {}", self.address_types)?;
        writeln!(f, "dial_info_detail_list:")?;
        for did in &self.dial_info_detail_list {
            writeln!(f, "{}", indent_all_string(did))?;
        }
        writeln!(f, "relay_kind: {}", self.relay_kind)?;
        Ok(())
    }
}

impl RelayInfo {
    pub fn new(
        timestamp: Timestamp,
        node_ids: NodeIdGroup,
        outbound_protocols: ProtocolTypeSet,
        address_types: AddressTypeSet,
        mut dial_info_detail_list: Vec<DialInfoDetail>,
        relay_kind: RelayKind,
    ) -> Self {
        dial_info_detail_list.sort();

        Self {
            timestamp,
            node_ids,
            outbound_protocols,
            address_types,
            dial_info_detail_list,
            relay_kind,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    pub fn node_ids(&self) -> &NodeIdGroup {
        &self.node_ids
    }
    pub fn outbound_protocols(&self) -> ProtocolTypeSet {
        self.outbound_protocols
    }
    pub fn address_types(&self) -> AddressTypeSet {
        self.address_types
    }
    pub fn relay_kind(&self) -> RelayKind {
        self.relay_kind
    }

    /// Compare this RelayInfo to another one
    /// Excludes the timestamp
    pub fn equivalent(&self, other: &RelayInfo) -> bool {
        let ani = self.node_ids();
        let bni = other.node_ids();
        let aop = self.outbound_protocols();
        let bop = other.outbound_protocols();
        let aat = self.address_types();
        let bat = other.address_types();
        let adids = self.dial_info_detail_list();
        let bdids = other.dial_info_detail_list();
        let aor = self.relay_kind();
        let bor = self.relay_kind();

        ani == bni && aop == bop && aat == bat && adids == bdids && aor == bor
    }
}

impl HasDialInfoDetailList for RelayInfo {
    fn dial_info_detail_list(&self) -> &[DialInfoDetail] {
        &self.dial_info_detail_list
    }

    fn has_sequencing_matched_dial_info(&self, sequencing: Sequencing) -> bool {
        // Check our dial info
        for did in self.dial_info_detail_list() {
            if sequencing.matches_ordering(did.dial_info.protocol_type().sequence_ordering()) {
                return true;
            }
        }

        false
    }
}
