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
