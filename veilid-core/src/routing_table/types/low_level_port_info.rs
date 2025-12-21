use super::*;

pub type LowLevelProtocolPort = (LowLevelProtocolType, AddressType, u16);
pub type LowLevelProtocolPorts = BTreeSet<LowLevelProtocolPort>;
pub type ProtocolToPortMapping = BTreeMap<(ProtocolType, AddressType), (LowLevelProtocolType, u16)>;
#[derive(Clone, Default, Debug)]
#[must_use]
pub struct LowLevelPortInfo {
    #[expect(dead_code)]
    pub low_level_protocol_ports: LowLevelProtocolPorts,
    pub protocol_to_port: ProtocolToPortMapping,
}
