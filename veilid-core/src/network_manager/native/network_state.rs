use super::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProtocolConfig {
    pub outbound: ProtocolTypeSet,
    pub inbound: ProtocolTypeSet,
    pub family_global: AddressTypeSet,
    pub family_local: AddressTypeSet,
    pub public_internet_capabilities: Vec<VeilidCapability>,
    pub local_network_capabilities: Vec<VeilidCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NetworkState {
    /// the calculated protocol configuration for inbound/outbound protocols
    pub protocol_config: ProtocolConfig,
    /// does our network have ipv4 on any interface?
    pub enable_ipv4: bool,
    /// does our network have ipv6 on any interface?
    pub enable_ipv6: bool,
    /// The interface addresses (network+mask) most recently seen
    pub interface_addresses: Arc<Vec<IfAddr>>,
}

impl Network {
    pub(super) fn last_network_state(&self) -> Option<Arc<NetworkState>> {
        self.inner.lock().network_state.clone()
    }

    pub(super) fn is_interface_address(&self, addr: IpAddr) -> bool {
        self.inner
            .lock()
            .network_state
            .as_ref()
            .unwrap()
            .interface_addresses
            .iter()
            .any(|x| x.ip() == addr)
    }

    pub(super) async fn refresh_network_state(&self) -> EyreResult<Option<Arc<NetworkState>>> {
        // Get last network state
        let last_network_state = self.inner.lock().network_state.clone();

        // Refresh network interfaces
        if !self
            .interfaces
            .refresh()
            .await
            .wrap_err("failed to refresh network interfaces")?
        {
            // Nothing changed
            return Ok(None);
        }

        let interface_addresses = self.interfaces.interface_addresses();

        if let Some(old_interface_addresses) =
            last_network_state.map(|x| x.interface_addresses.clone())
        {
            veilid_log!(self debug
                "Network interface addresses changed: \nFrom: {:?}\n  To: {:?}\n",
                old_interface_addresses, interface_addresses
            );
        } else {
            veilid_log!(self debug
                "Network interface addresses: \n  {:?}\n",
                interface_addresses
            );
        }

        // Determine if we have ipv4/ipv6 addresses
        let mut enable_ipv4 = false;
        let mut enable_ipv6 = false;

        for addr in interface_addresses.iter() {
            match addr {
                IfAddr::V4(_) => {
                    enable_ipv4 = true;
                }
                IfAddr::V6(_) => {
                    enable_ipv6 = true;
                }
            }
        }

        // Get protocol config
        let protocol_config = {
            let config = self.config();
            let mut inbound = ProtocolTypeSet::new();

            if config.network.protocol.udp.enabled {
                inbound.insert(ProtocolType::UDP);
            }
            if config.network.protocol.tcp.listen {
                inbound.insert(ProtocolType::TCP);
            }
            if config.network.protocol.ws.listen {
                inbound.insert(ProtocolType::WS);
            }
            #[cfg(feature = "enable-protocol-wss")]
            if config.network.protocol.wss.listen {
                inbound.insert(ProtocolType::WSS);
            }

            let mut outbound = ProtocolTypeSet::new();
            if config.network.protocol.udp.enabled {
                outbound.insert(ProtocolType::UDP);
            }
            if config.network.protocol.tcp.connect {
                outbound.insert(ProtocolType::TCP);
            }
            if config.network.protocol.ws.connect {
                outbound.insert(ProtocolType::WS);
            }
            #[cfg(feature = "enable-protocol-wss")]
            if config.network.protocol.wss.connect {
                outbound.insert(ProtocolType::WSS);
            }

            let mut family_global = AddressTypeSet::new();
            let mut family_local = AddressTypeSet::new();
            if enable_ipv4 {
                family_global.insert(AddressType::IPV4);
                family_local.insert(AddressType::IPV4);
            }
            if enable_ipv6 {
                family_global.insert(AddressType::IPV6);
                family_local.insert(AddressType::IPV6);
            }

            // set up the routing table's network config
            // if we have static public dialinfo, upgrade our network class
            let public_internet_capabilities = {
                PUBLIC_INTERNET_CAPABILITIES
                    .iter()
                    .copied()
                    .filter(|cap| !config.capabilities.disable.contains(cap))
                    .collect::<Vec<VeilidCapability>>()
            };
            let local_network_capabilities = {
                LOCAL_NETWORK_CAPABILITIES
                    .iter()
                    .copied()
                    .filter(|cap| !config.capabilities.disable.contains(cap))
                    .collect::<Vec<VeilidCapability>>()
            };

            ProtocolConfig {
                outbound,
                inbound,
                family_global,
                family_local,
                public_internet_capabilities,
                local_network_capabilities,
            }
        };

        let new_network_state = Some(Arc::new(NetworkState {
            protocol_config,
            enable_ipv4,
            enable_ipv6,
            interface_addresses,
        }));

        self.inner.lock().network_state = new_network_state.clone();

        Ok(new_network_state)
    }
}
