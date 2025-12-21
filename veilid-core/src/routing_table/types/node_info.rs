use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeInfo {
    timestamp: Timestamp,
    envelope_support: Vec<EnvelopeVersion>,
    crypto_info_list: Vec<CryptoInfo>,
    capabilities: Vec<VeilidCapability>,
    outbound_protocols: ProtocolTypeSet,
    address_types: AddressTypeSet,
    dial_info_detail_list: Vec<DialInfoDetail>,
    relay_info_list: Vec<RelayInfo>,
}

impl fmt::Display for NodeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "timestamp:          {}", self.timestamp)?;
        writeln!(f, "envelope_support:   {:?}", self.envelope_support)?;
        writeln!(f, "crypto_info_list:   {:?}", self.crypto_info_list)?;
        writeln!(f, "capabilities:       {:?}", self.capabilities)?;
        writeln!(f, "outbound_protocols: {:?}", self.outbound_protocols)?;
        writeln!(f, "address_types:      {:?}", self.address_types)?;
        writeln!(
            f,
            "dial_info_detail_list:\n{}",
            indent_all_string(&self.dial_info_detail_list.to_multiline_indexed_string())
        )?;

        writeln!(
            f,
            "relay_info_list:\n{}",
            indent_all_string(&self.relay_info_list.to_multiline_indexed_string())
        )?;

        Ok(())
    }
}

impl NodeInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timestamp: Timestamp,
        mut envelope_support: Vec<EnvelopeVersion>,
        mut crypto_info_list: Vec<CryptoInfo>,
        mut capabilities: Vec<VeilidCapability>,
        outbound_protocols: ProtocolTypeSet,
        address_types: AddressTypeSet,
        mut dial_info_detail_list: Vec<DialInfoDetail>,
        mut relay_info_list: Vec<RelayInfo>,
    ) -> Self {
        envelope_support.sort();
        crypto_info_list.sort();
        capabilities.sort();
        dial_info_detail_list.sort();
        relay_info_list.sort();
        Self {
            timestamp,
            envelope_support,
            crypto_info_list,
            capabilities,
            outbound_protocols,
            address_types,
            dial_info_detail_list,
            relay_info_list,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    pub fn envelope_support(&self) -> &[EnvelopeVersion] {
        &self.envelope_support
    }
    pub fn crypto_info_list(&self) -> &[CryptoInfo] {
        &self.crypto_info_list
    }
    pub fn capabilities(&self) -> &[VeilidCapability] {
        &self.capabilities
    }
    pub fn outbound_protocols(&self) -> ProtocolTypeSet {
        self.outbound_protocols
    }

    pub fn address_types(&self) -> AddressTypeSet {
        self.address_types
    }
    pub fn relay_info_list(&self) -> &[RelayInfo] {
        &self.relay_info_list
    }

    pub fn public_keys(&self) -> PublicKeyGroup {
        let mut pubkeys = PublicKeyGroup::new();
        for x in self.crypto_info_list.iter() {
            match x {
                #[cfg(feature = "enable-crypto-none")]
                CryptoInfo::NONE { public_key } => {
                    pubkeys.add(PublicKey::new(CRYPTO_KIND_NONE, public_key.clone()));
                }
                #[cfg(feature = "enable-crypto-vld0")]
                CryptoInfo::VLD0 { public_key } => {
                    pubkeys.add(PublicKey::new(CRYPTO_KIND_VLD0, public_key.clone()));
                } // #[cfg(feature = "enable-crypto-vld1")]
                  // CryptoInfo::VLD1 {
                  //     encapsulation_key,
                  //     signing_key,
                  // } => {
                  //     pubkeys.add(PublicKey::new(CRYPTO_KIND_VLD1, signing_key.clone()));
                  // }
            }
        }
        pubkeys
    }

    pub fn has_capability(&self, cap: VeilidCapability) -> bool {
        self.capabilities.contains(&cap)
    }
    pub fn has_all_capabilities(&self, capabilities: &[VeilidCapability]) -> bool {
        for cap in capabilities {
            if !self.has_capability(*cap) {
                return false;
            }
        }
        true
    }
    pub fn has_any_capabilities(&self, capabilities: &[VeilidCapability]) -> bool {
        if capabilities.is_empty() {
            return true;
        }
        for cap in capabilities {
            if self.has_capability(*cap) {
                return true;
            }
        }
        false
    }

    pub fn relay_ids(&self) -> Vec<NodeId> {
        let mut nids = vec![];
        for r in &self.relay_info_list {
            for nid in r.node_ids().iter() {
                nids.push(nid.clone());
            }
        }
        nids
    }

    pub fn has_any_dial_info(&self) -> bool {
        self.has_dial_info()
            || self
                .relay_info_list
                .iter()
                .find_map(|x| {
                    if !x.dial_info_detail_list().is_empty() {
                        Some(true)
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
    }

    /// Does this or any relay appear on the same ipblock?
    pub fn is_any_node_on_same_ipblock(&self, other: &NodeInfo, ip6_prefix_size: usize) -> bool {
        // our node vs their node
        if self.is_on_same_ipblock(other, ip6_prefix_size) {
            return true;
        }

        for other_relay_info in other.relay_info_list() {
            // our node vs their relay
            if self.is_on_same_ipblock(other_relay_info, ip6_prefix_size) {
                return true;
            }

            for our_relay_info in self.relay_info_list() {
                // our relay vs their relay
                if our_relay_info.is_on_same_ipblock(other_relay_info, ip6_prefix_size) {
                    return true;
                }
            }
        }

        for our_relay_info in self.relay_info_list() {
            // our relay vs their node
            if our_relay_info.is_on_same_ipblock(other, ip6_prefix_size) {
                return true;
            }
        }

        false
    }

    /// Build a map of protocols to low level ports
    /// This way we can get the set of protocols required to keep our NAT mapping alive for keepalive pings
    /// Only one protocol per low level protocol/port combination is required
    /// For example, if WS/WSS and TCP protocols are on the same low-level TCP port, only TCP keepalives will be required
    /// and we do not need to do WS/WSS keepalive as well. If they are on different ports, then we will need WS/WSS keepalives too.
    pub fn get_low_level_port_info(&self) -> LowLevelPortInfo {
        let mut low_level_protocol_ports =
            BTreeSet::<(LowLevelProtocolType, AddressType, u16)>::new();
        let mut protocol_to_port =
            BTreeMap::<(ProtocolType, AddressType), (LowLevelProtocolType, u16)>::new();

        for did in self.dial_info_detail_list() {
            low_level_protocol_ports.insert((
                did.dial_info.protocol_type().low_level_protocol_type(),
                did.dial_info.address_type(),
                did.dial_info.socket_address().port(),
            ));
            protocol_to_port.insert(
                (did.dial_info.protocol_type(), did.dial_info.address_type()),
                (
                    did.dial_info.protocol_type().low_level_protocol_type(),
                    did.dial_info.socket_address().port(),
                ),
            );
        }
        LowLevelPortInfo {
            low_level_protocol_ports,
            protocol_to_port,
        }
    }

    /// Compare this NodeInfo to another one
    /// Exclude the signature and timestamp and any other fields that are not
    /// semantically valuable
    pub fn equivalent(&self, other: &NodeInfo) -> bool {
        // Ignore
        //self.timestamp != other.timestamp ||
        if self.envelope_support != other.envelope_support
            || self.crypto_info_list != other.crypto_info_list
            || self.capabilities != other.capabilities
            || self.outbound_protocols != other.outbound_protocols
            || self.address_types != other.address_types
            || self.dial_info_detail_list != other.dial_info_detail_list
            || self.relay_info_list.len() != other.relay_info_list.len()
        {
            return false;
        }

        for x in 0..self.relay_info_list.len() {
            if !self.relay_info_list[x].equivalent(&other.relay_info_list[x]) {
                return false;
            }
        }

        true
    }

    #[cfg(feature = "geolocation")]
    pub fn get_geolocation_info(&self, routing_domain: RoutingDomain) -> GeolocationInfo {
        if routing_domain != RoutingDomain::PublicInternet {
            // Country code is irrelevant for local network
            return GeolocationInfo::new(None, vec![]);
        }

        let relay_country_codes: Vec<Option<CountryCode>> = self
            .relay_info_list
            .iter()
            .map(|x| x.get_country_code())
            .collect();

        GeolocationInfo::new(self.get_country_code(), relay_country_codes)
    }
}

impl HasDialInfoDetailList for NodeInfo {
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
        // Check our relays
        self.relay_info_list
            .iter()
            .find_map(|relay_info| {
                if relay_info.has_sequencing_matched_dial_info(sequencing) {
                    return Some(true);
                }
                None
            })
            .unwrap_or_default()
    }
}
