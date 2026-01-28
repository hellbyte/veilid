use super::*;

fourcc_type!(VeilidCapability);
pub const VEILID_CAPABILITY_ROUTE: VeilidCapability = VeilidCapability::new(*b"ROUT");
#[cfg(feature = "unstable-tunnels")]
pub const VEILID_CAPABILITY_TUNNEL: VeilidCapability = VeilidCapability::new(*b"TUNL");
pub const VEILID_CAPABILITY_SIGNAL: VeilidCapability = VeilidCapability::new(*b"SGNL");
pub const VEILID_CAPABILITY_RELAY: VeilidCapability = VeilidCapability::new(*b"RLAY");
pub const VEILID_CAPABILITY_VALIDATE_DIAL_INFO: VeilidCapability = VeilidCapability::new(*b"DIAL");
pub const VEILID_CAPABILITY_DHT: VeilidCapability = VeilidCapability::new(*b"DHTV");
pub const VEILID_CAPABILITY_APPMESSAGE: VeilidCapability = VeilidCapability::new(*b"APPM");
#[cfg(feature = "unstable-blockstore")]
pub const VEILID_CAPABILITY_BLOCKSTORE: VeilidCapability = VeilidCapability::new(*b"BLOC");

pub const DISTANCE_METRIC_CAPABILITIES: &[VeilidCapability] = &[VEILID_CAPABILITY_DHT];
pub const CONNECTIVITY_CAPABILITIES: &[VeilidCapability] = &[
    VEILID_CAPABILITY_RELAY,
    VEILID_CAPABILITY_SIGNAL,
    VEILID_CAPABILITY_ROUTE,
    VEILID_CAPABILITY_VALIDATE_DIAL_INFO,
];

cfg_if! {
    if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
        pub const PUBLIC_INTERNET_CAPABILITIES: &[VeilidCapability] = &[
            VEILID_CAPABILITY_ROUTE,
            #[cfg(feature = "unstable-tunnels")]
            VEILID_CAPABILITY_TUNNEL,
            VEILID_CAPABILITY_SIGNAL,
            VEILID_CAPABILITY_DHT,
            VEILID_CAPABILITY_APPMESSAGE,
            #[cfg(feature = "unstable-blockstore")]
            VEILID_CAPABILITY_BLOCKSTORE,
        ];

        pub const LOCAL_NETWORK_CAPABILITIES: &[VeilidCapability] = &[VEILID_CAPABILITY_APPMESSAGE];
    } else {
        pub const PUBLIC_INTERNET_CAPABILITIES: &[VeilidCapability] = &[
            VEILID_CAPABILITY_ROUTE,
            #[cfg(feature = "unstable-tunnels")]
            VEILID_CAPABILITY_TUNNEL,
            VEILID_CAPABILITY_SIGNAL,
            VEILID_CAPABILITY_RELAY,
            VEILID_CAPABILITY_VALIDATE_DIAL_INFO,
            VEILID_CAPABILITY_DHT,
            VEILID_CAPABILITY_APPMESSAGE,
            #[cfg(feature = "unstable-blockstore")]
            VEILID_CAPABILITY_BLOCKSTORE,
        ];

        pub const LOCAL_NETWORK_CAPABILITIES: &[VeilidCapability] =
            &[VEILID_CAPABILITY_RELAY, VEILID_CAPABILITY_APPMESSAGE];
    }
}

pub const MAX_CAPABILITIES: usize = 64;
