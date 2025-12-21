use super::*;

/// The completion state of the routing domain on its path to publishability as PeerInfo
#[derive(Debug, Clone)]
pub enum RoutingDomainState {
    /// No network setup (no outbound protocols, address types, or capabilities). No dialinfo confirmation.
    Invalid,
    /// Network setup, but no dialinfo confirmation.
    NeedsDialInfoConfirmation,
    /// Network setup, dialinfo confirmation, but no address types or outbound protocols are enabled.
    Unusable,
    /// Network setup, dialinfo confirmation, address types + protocol types are valid, but 1+ relays must be selected because of missing dialinfo.
    NeedsRelays { relay_status: RelayStatus },
    /// Network setup, dialinfo confirmation, address types + protocol types are valid, and all relays needed are selected.
    ReadyToPublish { relay_status: RelayStatus },
}
