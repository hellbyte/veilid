use crate::*;

cfg_if::cfg_if! {
    if #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))] {
        use sysinfo::System;
        use lazy_static::*;
        use directories::ProjectDirs;

        lazy_static! {
            static ref SYSTEM:System = {
                sysinfo::System::new_with_specifics(
                    sysinfo::RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::everything()),
                )
            };
        }
    }
}

/// Enable and configure HTTPS access to the Veilid node.
///
/// ```yaml
/// https:
///     enabled: false
///     listen_address: ':5150'
///     path: 'app'
///     url: 'https://localhost:5150'
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigHTTPS {
    pub enabled: bool,
    pub listen_address: String,
    pub path: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub url: Option<String>, // Fixed URL is not optional for TLS-based protocols and is dynamically validated
}
impl Default for VeilidConfigHTTPS {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_address: String::from(""),
            path: String::from("app"),
            url: None,
        }
    }
}

/// Enable and configure HTTP access to the Veilid node.
///
/// ```yaml
/// http:
///     enabled: false
///     listen_address: ':5150'
///     path: 'app"
///     url: 'https://localhost:5150'
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigHTTP {
    pub enabled: bool,
    pub listen_address: String,
    pub path: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub url: Option<String>,
}

impl Default for VeilidConfigHTTP {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_address: String::from(""),
            path: String::from("app"),
            url: None,
        }
    }
}

/// Enable and configure UDP.
///
/// ```yaml
/// udp:
///     enabled: true
///     socket_pool_size: 0
///     listen_address: ':5150'
///     public_address: ''
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigUDP {
    pub enabled: bool,
    pub socket_pool_size: u32,
    pub listen_address: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub public_address: Option<String>,
}

impl Default for VeilidConfigUDP {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
                let enabled = false;
            } else {
                let enabled = true;
            }
        }
        Self {
            enabled,
            socket_pool_size: 0,
            listen_address: String::from(""),
            public_address: None,
        }
    }
}

/// Enable and configure TCP.
///
/// ```yaml
/// tcp:
///     connect: true
///     listen: true
///     max_connections: 32
///     listen_address: ':5150'
///     public_address: ''
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigTCP {
    pub connect: bool,
    pub listen: bool,
    pub max_connections: u32,
    pub listen_address: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub public_address: Option<String>,
}

impl Default for VeilidConfigTCP {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
                let connect = false;
                let listen = false;
            } else {
                let connect = true;
                let listen = true;
            }
        }
        Self {
            connect,
            listen,
            max_connections: 32,
            listen_address: String::from(""),
            public_address: None,
        }
    }
}

/// Enable and configure Web Sockets.
///
/// ```yaml
/// ws:
///     connect: true
///     listen: true
///     max_connections: 32
///     listen_address: ':5150'
///     path: 'ws'
///     url: 'ws://localhost:5150/ws'
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigWS {
    pub connect: bool,
    pub listen: bool,
    pub max_connections: u32,
    pub listen_address: String,
    pub path: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub url: Option<String>,
}

impl Default for VeilidConfigWS {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
                let connect = true;
                let listen = false;
            } else {
                let connect = true;
                let listen = true;
            }
        }
        Self {
            connect,
            listen,
            max_connections: 32,
            listen_address: String::from(""),
            path: String::from("ws"),
            url: None,
        }
    }
}

/// Enable and configure Secure Web Sockets.
///
/// ```yaml
/// wss:
///     connect: true
///     listen: false
///     max_connections: 32
///     listen_address: ':5150'
///     path: 'ws'
///     url: ''
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
#[cfg(feature = "enable-protocol-wss")]
pub struct VeilidConfigWSS {
    pub connect: bool,
    pub listen: bool,
    pub max_connections: u32,
    pub listen_address: String,
    pub path: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub url: Option<String>, // Fixed URL is not optional for TLS-based protocols and is dynamically validated
}

#[cfg(feature = "enable-protocol-wss")]
impl Default for VeilidConfigWSS {
    fn default() -> Self {
        Self {
            connect: true,
            listen: false,
            max_connections: 32,
            listen_address: String::from(""),
            path: String::from("ws"),
            url: None,
        }
    }
}

/// Configure Network Protocols.
///
/// Veilid can communicate over UDP, TCP, and Web Sockets.
///
/// All protocols are available by default, and the Veilid node will
/// sort out which protocol is used for each peer connection.
///
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigProtocol {
    pub udp: VeilidConfigUDP,
    pub tcp: VeilidConfigTCP,
    pub ws: VeilidConfigWS,
    #[cfg(feature = "enable-protocol-wss")]
    pub wss: VeilidConfigWSS,
}

/// Privacy preferences for routes.
///
/// ```yaml
/// privacy:
///     require_inbound_relay: false
///     country_code_denylist: [] # only with `--features=geolocation`
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigPrivacy {
    pub require_inbound_relay: bool,
    #[cfg(feature = "geolocation")]
    pub country_code_denylist: Vec<CountryCode>,
}

/// Virtual networking client support for testing/simulation purposes
///
/// ```yaml
/// virtual_network:
///     enabled: false
///     server_address: ""
/// ```
#[cfg(feature = "virtual-network")]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigVirtualNetwork {
    pub enabled: bool,
    pub server_address: String,
}

/// Configure TLS.
///
/// ```yaml
/// tls:
///     certificate_path: /path/to/cert
///     private_key_path: /path/to/private/key
///     connection_initial_timeout_ms: 2000
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigTLS {
    pub certificate_path: String,
    pub private_key_path: String,
    pub connection_initial_timeout_ms: u32,
}

impl Default for VeilidConfigTLS {
    fn default() -> Self {
        Self {
            certificate_path: "".to_string(),
            private_key_path: "".to_string(),
            connection_initial_timeout_ms: 2000,
        }
    }
}

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    allow(unused_variables)
)]
#[must_use]
pub fn get_default_ssl_directory(
    program_name: &str,
    organization: &str,
    qualifier: &str,
    sub_path: &str,
) -> String {
    cfg_if::cfg_if! {
        if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
            "".to_owned()
        } else {
            use std::path::PathBuf;
            ProjectDirs::from(qualifier, organization, program_name)
                .map(|dirs| dirs.data_local_dir().join("ssl").join(sub_path))
                .unwrap_or_else(|| PathBuf::from("./ssl").join(sub_path))
                .to_string_lossy()
                .into()
        }
    }
}

/// Configure the Distributed Hash Table (DHT).
/// Defaults should be used here unless you are absolutely sure you know what you're doing.
/// If you change the count/fanout/timeout parameters, you may render your node inoperable
/// for correct DHT operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigDHT {
    pub max_find_node_count: u32,
    pub resolve_node_timeout_ms: u32,
    pub resolve_node_count: u32,
    pub resolve_node_fanout: u32,
    pub get_value_timeout_ms: u32,
    pub get_value_count: u32,
    pub get_value_fanout: u32,
    pub set_value_timeout_ms: u32,
    pub set_value_count: u32,
    pub set_value_fanout: u32,
    pub consensus_width: u32,
    pub min_peer_count: u32,
    pub min_peer_refresh_time_ms: u32,
    pub validate_dial_info_receipt_time_ms: u32,
    pub local_subkey_cache_size: u32,
    pub local_max_subkey_cache_memory_mb: u32,
    pub remote_subkey_cache_size: u32,
    pub remote_max_records: u32,
    pub remote_max_subkey_cache_memory_mb: u32,
    pub remote_max_storage_space_mb: u32,
    pub public_watch_limit: u32,
    pub member_watch_limit: u32,
    pub max_watch_expiration_ms: u32,
    pub public_transaction_limit: u32,
    pub member_transaction_limit: u32,
}

impl Default for VeilidConfigDHT {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
                let local_subkey_cache_size = 128;
                let local_max_subkey_cache_memory_mb = 256;
                let remote_subkey_cache_size = 64;
                let remote_max_records = 64;
                let remote_max_subkey_cache_memory_mb = 256;
                let remote_max_storage_space_mb = 128;
            } else {
                let local_subkey_cache_size = 1024;
                let local_max_subkey_cache_memory_mb = if sysinfo::IS_SUPPORTED_SYSTEM {
                    (SYSTEM.total_memory() / 32u64 / (1024u64 * 1024u64)) as u32
                } else {
                    256
                };
                let remote_subkey_cache_size = 128;
                let remote_max_records = 128;
                let remote_max_subkey_cache_memory_mb = if sysinfo::IS_SUPPORTED_SYSTEM {
                    (SYSTEM.total_memory() / 32u64 / (1024u64 * 1024u64)) as u32
                } else {
                    256
                };
                let remote_max_storage_space_mb = 256;
            }
        }

        Self {
            max_find_node_count: 20,
            resolve_node_timeout_ms: 10000,
            resolve_node_count: 1,
            resolve_node_fanout: 5,
            get_value_timeout_ms: 10000,
            get_value_count: 3,
            get_value_fanout: 5,
            set_value_timeout_ms: 10000,
            set_value_count: 5,
            set_value_fanout: 5,
            consensus_width: 10,
            min_peer_count: 20,
            min_peer_refresh_time_ms: 60000,
            validate_dial_info_receipt_time_ms: 2000,
            local_subkey_cache_size,
            local_max_subkey_cache_memory_mb,
            remote_subkey_cache_size,
            remote_max_records,
            remote_max_subkey_cache_memory_mb,
            remote_max_storage_space_mb,
            public_watch_limit: 32,
            member_watch_limit: 8,
            max_watch_expiration_ms: 600000,
            public_transaction_limit: 4,
            member_transaction_limit: 1,
        }
    }
}

/// Configure RPC.
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigRPC {
    pub concurrency: u32,
    pub queue_size: u32,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub max_timestamp_behind_ms: Option<u32>,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub max_timestamp_ahead_ms: Option<u32>,
    pub timeout_ms: u32,
    pub max_route_hop_count: u8,
    pub default_route_hop_count: u8,
}

impl Default for VeilidConfigRPC {
    fn default() -> Self {
        Self {
            concurrency: 0,
            queue_size: 1024,
            max_timestamp_behind_ms: Some(10000),
            max_timestamp_ahead_ms: Some(10000),
            timeout_ms: 5000,
            max_route_hop_count: 4,
            default_route_hop_count: 1,
        }
    }
}

/// Configure the network routing table.
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigRoutingTable {
    #[schemars(with = "Vec<String>")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        tsify(type = "string[]")
    )]
    pub public_keys: PublicKeyGroup,
    #[schemars(with = "Vec<String>")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        tsify(type = "string[]")
    )]
    pub secret_keys: SecretKeyGroup,
    pub bootstrap: Vec<String>,
    #[schemars(with = "Vec<String>")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        tsify(type = "string[]")
    )]
    pub bootstrap_keys: Vec<PublicKey>,
    pub limit_over_attached: u32,
    pub limit_fully_attached: u32,
    pub limit_attached_strong: u32,
    pub limit_attached_good: u32,
    pub limit_attached_weak: u32,
    // xxx pub enable_public_internet: bool,
    // xxx pub enable_local_network: bool,
}

impl Default for VeilidConfigRoutingTable {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
                let bootstrap = vec!["ws://bootstrap-v1.veilid.net:5150/ws".to_string()];
            } else {
                let bootstrap = vec!["bootstrap-v1.veilid.net".to_string()];
            }
        }
        let bootstrap_keys = vec![
            // Primary Veilid Foundation bootstrap signing key
            PublicKey::from_str("VLD0:Vj0lKDdUQXmQ5Ol1SZdlvXkBHUccBcQvGLN9vbLSI7k").unwrap_or_log(),
            // Secondary Veilid Foundation bootstrap signing key
            PublicKey::from_str("VLD0:QeQJorqbXtC7v3OlynCZ_W3m76wGNeB5NTF81ypqHAo").unwrap_or_log(),
            // Backup Veilid Foundation bootstrap signing key
            PublicKey::from_str("VLD0:QNdcl-0OiFfYVj9331XVR6IqZ49NG-E18d5P7lwi4TA").unwrap_or_log(),
        ];

        Self {
            public_keys: PublicKeyGroup::default(),
            secret_keys: SecretKeyGroup::default(),
            bootstrap,
            bootstrap_keys,
            limit_over_attached: 64,
            limit_fully_attached: 32,
            limit_attached_strong: 16,
            limit_attached_good: 8,
            limit_attached_weak: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigNetwork {
    pub connection_initial_timeout_ms: u32,
    pub connection_inactivity_timeout_ms: u32,
    pub max_connections_per_ip4: u32,
    pub max_connections_per_ip6_prefix: u32,
    pub max_connections_per_ip6_prefix_size: u32,
    pub max_connection_frequency_per_min: u32,
    pub client_allowlist_timeout_ms: u32,
    pub reverse_connection_receipt_time_ms: u32,
    pub hole_punch_receipt_time_ms: u32,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub network_key_password: Option<String>,
    pub routing_table: VeilidConfigRoutingTable,
    pub rpc: VeilidConfigRPC,
    pub dht: VeilidConfigDHT,
    pub upnp: bool,
    pub detect_address_changes: Option<bool>,
    pub restricted_nat_retries: u32,
    pub tls: VeilidConfigTLS,
    pub protocol: VeilidConfigProtocol,
    pub privacy: VeilidConfigPrivacy,
    #[cfg(feature = "virtual-network")]
    pub virtual_network: VeilidConfigVirtualNetwork,
}

impl Default for VeilidConfigNetwork {
    fn default() -> Self {
        Self {
            connection_initial_timeout_ms: 2000,
            connection_inactivity_timeout_ms: 60000,
            max_connections_per_ip4: 32,
            max_connections_per_ip6_prefix: 32,
            max_connections_per_ip6_prefix_size: 56,
            max_connection_frequency_per_min: 128,
            client_allowlist_timeout_ms: 300000,
            reverse_connection_receipt_time_ms: 5000,
            hole_punch_receipt_time_ms: 5000,
            network_key_password: None,
            routing_table: VeilidConfigRoutingTable::default(),
            rpc: VeilidConfigRPC::default(),
            dht: VeilidConfigDHT::default(),
            upnp: true,
            detect_address_changes: Some(true),
            restricted_nat_retries: 0,
            tls: VeilidConfigTLS::default(),
            protocol: VeilidConfigProtocol::default(),
            privacy: VeilidConfigPrivacy::default(),
            #[cfg(feature = "virtual-network")]
            virtual_network: VeilidConfigVirtualNetwork::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigTableStore {
    pub directory: String,
    pub delete: bool,
}

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    allow(unused_variables)
)]
#[must_use]
fn get_default_store_path(
    program_name: &str,
    organization: &str,
    qualifier: &str,
    store_type: &str,
) -> String {
    cfg_if::cfg_if! {
        if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
            "".to_owned()
        } else {
            use std::path::PathBuf;
            ProjectDirs::from(qualifier, organization, program_name)
                .map(|dirs| dirs.data_local_dir().to_path_buf())
                .unwrap_or_else(|| PathBuf::from("./"))
                .join(store_type)
                .to_string_lossy()
                .into()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigBlockStore {
    pub directory: String,
    pub delete: bool,
}

impl Default for VeilidConfigBlockStore {
    fn default() -> Self {
        Self {
            directory: "".to_string(),
            delete: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigProtectedStore {
    pub allow_insecure_fallback: bool,
    pub always_use_insecure_storage: bool,
    pub directory: String,
    pub delete: bool,
    pub device_encryption_key_password: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub new_device_encryption_key_password: Option<String>,
}

impl Default for VeilidConfigProtectedStore {
    fn default() -> Self {
        Self {
            allow_insecure_fallback: false,
            always_use_insecure_storage: false,
            directory: "".to_string(),
            delete: false,
            device_encryption_key_password: "".to_owned(),
            new_device_encryption_key_password: None,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfigCapabilities {
    pub disable: Vec<VeilidCapability>,
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(namespace, into_wasm_abi, from_wasm_abi)
)]
#[must_use]
#[derive(Default)]
pub enum VeilidConfigLogLevel {
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<VeilidLogLevel> for VeilidConfigLogLevel {
    fn from(value: VeilidLogLevel) -> Self {
        match value {
            VeilidLogLevel::Error => Self::Error,
            VeilidLogLevel::Warn => Self::Warn,
            VeilidLogLevel::Info => Self::Info,
            VeilidLogLevel::Debug => Self::Debug,
            VeilidLogLevel::Trace => Self::Trace,
        }
    }
}

impl From<Option<VeilidLogLevel>> for VeilidConfigLogLevel {
    fn from(value: Option<VeilidLogLevel>) -> Self {
        match value {
            None => Self::Off,
            Some(VeilidLogLevel::Error) => Self::Error,
            Some(VeilidLogLevel::Warn) => Self::Warn,
            Some(VeilidLogLevel::Info) => Self::Info,
            Some(VeilidLogLevel::Debug) => Self::Debug,
            Some(VeilidLogLevel::Trace) => Self::Trace,
        }
    }
}

impl From<tracing::level_filters::LevelFilter> for VeilidConfigLogLevel {
    fn from(value: tracing::level_filters::LevelFilter) -> Self {
        match value {
            tracing::level_filters::LevelFilter::OFF => Self::Off,
            tracing::level_filters::LevelFilter::ERROR => Self::Error,
            tracing::level_filters::LevelFilter::WARN => Self::Warn,
            tracing::level_filters::LevelFilter::INFO => Self::Info,
            tracing::level_filters::LevelFilter::DEBUG => Self::Debug,
            tracing::level_filters::LevelFilter::TRACE => Self::Trace,
        }
    }
}

impl From<VeilidConfigLogLevel> for tracing::level_filters::LevelFilter {
    fn from(val: VeilidConfigLogLevel) -> Self {
        match val {
            VeilidConfigLogLevel::Off => tracing::level_filters::LevelFilter::OFF,
            VeilidConfigLogLevel::Error => tracing::level_filters::LevelFilter::ERROR,
            VeilidConfigLogLevel::Warn => tracing::level_filters::LevelFilter::WARN,
            VeilidConfigLogLevel::Info => tracing::level_filters::LevelFilter::INFO,
            VeilidConfigLogLevel::Debug => tracing::level_filters::LevelFilter::DEBUG,
            VeilidConfigLogLevel::Trace => tracing::level_filters::LevelFilter::TRACE,
        }
    }
}

impl From<tracing::log::LevelFilter> for VeilidConfigLogLevel {
    fn from(value: tracing::log::LevelFilter) -> Self {
        match value {
            tracing::log::LevelFilter::Off => Self::Off,
            tracing::log::LevelFilter::Error => Self::Error,
            tracing::log::LevelFilter::Warn => Self::Warn,
            tracing::log::LevelFilter::Info => Self::Info,
            tracing::log::LevelFilter::Debug => Self::Debug,
            tracing::log::LevelFilter::Trace => Self::Trace,
        }
    }
}

impl From<VeilidConfigLogLevel> for tracing::log::LevelFilter {
    fn from(val: VeilidConfigLogLevel) -> Self {
        match val {
            VeilidConfigLogLevel::Off => tracing::log::LevelFilter::Off,
            VeilidConfigLogLevel::Error => tracing::log::LevelFilter::Error,
            VeilidConfigLogLevel::Warn => tracing::log::LevelFilter::Warn,
            VeilidConfigLogLevel::Info => tracing::log::LevelFilter::Info,
            VeilidConfigLogLevel::Debug => tracing::log::LevelFilter::Debug,
            VeilidConfigLogLevel::Trace => tracing::log::LevelFilter::Trace,
        }
    }
}

impl TryFrom<&str> for VeilidConfigLogLevel {
    type Error = VeilidAPIError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for VeilidConfigLogLevel {
    type Error = VeilidAPIError;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&String> for VeilidConfigLogLevel {
    type Error = VeilidAPIError;

    fn try_from(value: &String) -> Result<Self, <Self as TryFrom<&String>>::Error> {
        Self::from_str(value.as_str())
    }
}

impl FromStr for VeilidConfigLogLevel {
    type Err = VeilidAPIError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "off" => Self::Off,
            "error" => Self::Error,
            "warn" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => {
                apibail_invalid_argument!("invalid VeilidConfigLogLevel string", "s", s);
            }
        })
    }
}
impl fmt::Display for VeilidConfigLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match self {
            Self::Off => "Off",
            Self::Error => "Error",
            Self::Warn => "Warn",
            Self::Info => "Info",
            Self::Debug => "Debug",
            Self::Trace => "Trace",
        };
        write!(f, "{}", text)
    }
}

/// Top level of the Veilid configuration tree
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct VeilidConfig {
    /// An identifier used to describe the program using veilid-core.
    /// Used to partition storage locations in places like the ProtectedStore.
    /// Must be non-empty and a valid filename for all Veilid-capable systems, which means
    /// no backslashes or forward slashes in the name. Stick to a-z,0-9,_ and space and you should be fine.
    ///
    /// Caution: If you change this string, there is no migration support. Your app's protected store and
    /// table store will very likely experience data loss. Pick a program name and stick with it. This is
    /// not a 'visible' identifier and it should uniquely identify your application.
    pub program_name: String,
    /// To run multiple Veilid nodes within the same application, either through a single process running
    /// api_startup/api_startup_json multiple times, or your application running mulitple times side-by-side
    /// there needs to be a key used to partition the application's storage (in the TableStore, ProtectedStore, etc).
    /// An empty value here is the default, but if you run multiple veilid nodes concurrently, you should set this
    /// to a string that uniquely identifies this -instance- within the same 'program_name'.
    /// Must be a valid filename for all Veilid-capable systems, which means no backslashes or forward slashes
    /// in the name. Stick to a-z,0-9,_ and space and you should be fine.
    pub namespace: String,
    /// Capabilities to enable for your application/node
    pub capabilities: VeilidConfigCapabilities,
    /// Configuring the protected store (keychain/keyring/etc)
    pub protected_store: VeilidConfigProtectedStore,
    /// Configuring the table store (persistent encrypted database)
    pub table_store: VeilidConfigTableStore,
    /// Configuring the block store (storage of large content-addressable content)
    pub block_store: VeilidConfigBlockStore,
    /// Configuring how Veilid interacts with the low level network
    pub network: VeilidConfigNetwork,
}

impl VeilidConfig {
    /// Create a new 'VeilidConfig' for use with `setup_from_config`
    /// Should match the application bundle name if used elsewhere in the format:
    /// `qualifier.organization.program_name` - for example `org.veilid.veilidchat`
    ///
    /// The 'bundle name' will be used when choosing the default storage location for the
    /// application in a platform-dependent fashion, unless 'storage_directory' is
    /// specified to override this location
    ///
    /// * `program_name` - Pick a program name and do not change it from release to release,
    ///   see `VeilidConfig::program_name` for details.
    /// * `organization_name` - Similar to program_name, but for the organization publishing this app
    /// * `qualifier` - Suffix for the application bundle name
    /// * `storage_directory` - Override for the path where veilid-core stores its content
    ///   such as the table store, protected store, and block store
    /// * `config_directory` - Override for the path where veilid-core can retrieve extra configuration files
    ///   such as certificates and keys
    pub fn new(
        program_name: &str,
        organization: &str,
        qualifier: &str,
        storage_directory: Option<&str>,
        config_directory: Option<&str>,
    ) -> Self {
        let mut out = Self {
            program_name: program_name.to_owned(),
            ..Default::default()
        };

        if let Some(storage_directory) = storage_directory {
            out.protected_store.directory = (std::path::PathBuf::from(storage_directory)
                .join("protected_store"))
            .to_string_lossy()
            .to_string();
            out.table_store.directory = (std::path::PathBuf::from(storage_directory)
                .join("table_store"))
            .to_string_lossy()
            .to_string();
            out.block_store.directory = (std::path::PathBuf::from(storage_directory)
                .join("block_store"))
            .to_string_lossy()
            .to_string();
        } else {
            out.protected_store.directory =
                get_default_store_path(program_name, organization, qualifier, "protected_store");
            out.table_store.directory =
                get_default_store_path(program_name, organization, qualifier, "table_store");
            out.block_store.directory =
                get_default_store_path(program_name, organization, qualifier, "block_store");
        }

        if let Some(config_directory) = config_directory {
            out.network.tls.certificate_path = (std::path::PathBuf::from(config_directory)
                .join("ssl/certs/server.crt"))
            .to_string_lossy()
            .to_string();
            out.network.tls.private_key_path = (std::path::PathBuf::from(config_directory)
                .join("ssl/keys/server.key"))
            .to_string_lossy()
            .to_string();
        } else {
            out.network.tls.certificate_path = get_default_ssl_directory(
                program_name,
                organization,
                qualifier,
                "certs/server.crt",
            );
            out.network.tls.private_key_path =
                get_default_ssl_directory(program_name, organization, qualifier, "keys/server.key");
        }

        out
    }

    #[must_use]
    pub fn safe(&self) -> Arc<VeilidConfig> {
        let mut safe_cfg = self.clone();

        // Remove secrets
        safe_cfg.network.routing_table.secret_keys = SecretKeyGroup::new();
        "".clone_into(&mut safe_cfg.protected_store.device_encryption_key_password);
        safe_cfg.protected_store.new_device_encryption_key_password = None;

        Arc::new(safe_cfg)
    }

    pub fn get_key_json(&self, key: &str, pretty: bool) -> VeilidAPIResult<String> {
        // Generate json from whole config
        let jc = serde_json::to_string(self).map_err(VeilidAPIError::generic)?;
        let jvc = json::parse(&jc).map_err(VeilidAPIError::generic)?;

        // Find requested subkey
        if key.is_empty() {
            Ok(if pretty {
                jvc.pretty(2)
            } else {
                jvc.to_string()
            })
        } else {
            // Split key into path parts
            let keypath: Vec<&str> = key.split('.').collect();
            let mut out = &jvc;
            for k in keypath {
                if !out.has_key(k) {
                    apibail_parse_error!(format!("invalid subkey in key '{}'", key), k);
                }
                out = &out[k];
            }
            Ok(if pretty {
                out.pretty(2)
            } else {
                out.to_string()
            })
        }
    }

    fn validate_program_name(program_name: &str) -> VeilidAPIResult<()> {
        if program_name.is_empty() {
            apibail_generic!("Program name must not be empty in 'program_name'");
        }
        if !sanitize_filename::is_sanitized_with_options(
            program_name,
            sanitize_filename::OptionsForCheck {
                windows: true,
                truncate: true,
            },
        ) {
            apibail_generic!("'program_name' must not be an invalid filename");
        }
        Ok(())
    }

    fn validate_namespace(namespace: &str) -> VeilidAPIResult<()> {
        if namespace.is_empty() {
            return Ok(());
        }
        if !sanitize_filename::is_sanitized_with_options(
            namespace,
            sanitize_filename::OptionsForCheck {
                windows: true,
                truncate: true,
            },
        ) {
            apibail_generic!("'namespace' must not be an invalid filename");
        }

        Ok(())
    }

    pub fn validate(&self) -> VeilidAPIResult<()> {
        Self::validate_program_name(&self.program_name)?;
        Self::validate_namespace(&self.namespace)?;

        // if inner.network.protocol.udp.enabled {
        //     // Validate UDP settings
        // }
        if self.network.protocol.tcp.listen {
            // Validate TCP settings
            if self.network.protocol.tcp.max_connections == 0 {
                apibail_generic!("TCP max connections must be > 0 in config key 'network.protocol.tcp.max_connections'");
            }
        }
        if self.network.protocol.ws.listen {
            // Validate WS settings
            if self.network.protocol.ws.max_connections == 0 {
                apibail_generic!("WS max connections must be > 0 in config key 'network.protocol.ws.max_connections'");
            }
        }
        #[cfg(feature = "enable-protocol-wss")]
        if self.network.protocol.wss.listen {
            // Validate WSS settings
            if self.network.protocol.wss.max_connections == 0 {
                apibail_generic!("WSS max connections must be > 0 in config key 'network.protocol.wss.max_connections'");
            }
            if self
                .network
                .protocol
                .wss
                .url
                .as_ref()
                .map(|u| u.is_empty())
                .unwrap_or_default()
            {
                apibail_generic!(
                    "WSS URL must be specified in config key 'network.protocol.wss.url'"
                );
            }
        }
        if self.network.rpc.max_route_hop_count == 0 {
            apibail_generic!(
                "max route hop count must be >= 1 in 'network.rpc.max_route_hop_count'"
            );
        }
        if self.network.rpc.max_route_hop_count > 5 {
            apibail_generic!(
                "max route hop count must be <= 5 in 'network.rpc.max_route_hop_count'"
            );
        }
        if self.network.rpc.default_route_hop_count == 0 {
            apibail_generic!(
                "default route hop count must be >= 1 in 'network.rpc.default_route_hop_count'"
            );
        }
        if self.network.rpc.default_route_hop_count > self.network.rpc.max_route_hop_count {
            apibail_generic!(
                "default route hop count must be <= max route hop count in 'network.rpc.default_route_hop_count <= network.rpc.max_route_hop_count'"
            );
        }
        if self.network.rpc.queue_size < 256 {
            apibail_generic!("rpc queue size must be >= 256 in 'network.rpc.queue_size'");
        }
        if self.network.rpc.timeout_ms < 1000 {
            apibail_generic!("rpc timeout must be >= 1000 in 'network.rpc.timeout_ms'");
        }
        if self.network.dht.consensus_width < self.network.dht.set_value_count {
            apibail_generic!(
                "consensus width must be >= set value count in 'network.dht.consensus_width'"
            );
        }
        if self.network.dht.get_value_count <= (self.network.dht.set_value_count / 2) {
            apibail_generic!("get consensus count must be >= (set value count / 2) in 'network.dht.get_value_count'");
        }
        if self.network.dht.get_value_fanout < 1 {
            apibail_generic!("get value fanout must be >= 1 in 'network.dht.get_value_fanout'");
        }
        if self.network.dht.set_value_fanout < 1 {
            apibail_generic!("set value fanout must be >= 1 in 'network.dht.set_value_fanout'");
        }
        if self.network.dht.get_value_timeout_ms < (2 * self.network.rpc.timeout_ms) {
            apibail_generic!("get value timeout must be >= (2 * the rpc timeout) in 'network.dht.get_value_timeout_ms'");
        }
        if self.network.dht.set_value_timeout_ms < (2 * self.network.rpc.timeout_ms) {
            apibail_generic!("set value timeout must be >= (2 * the rpc timeout) in 'network.dht.set_value_timeout_ms'");
        }

        if self.network.dht.public_watch_limit < 1 {
            apibail_generic!("public watch limit must be >= 1 in 'network.dht.public_watch_limit'");
        }
        if self.network.dht.member_watch_limit < 1 {
            apibail_generic!("member watch limit must be >= 1 in 'network.dht.member_watch_limit'");
        }
        if self.network.dht.max_watch_expiration_ms < (2 * self.network.rpc.timeout_ms) {
            apibail_generic!("max watch expiration must be >= (2 * rpc timeout) 'network.dht.max_watch_expiration_ms'");
        }
        if self.network.dht.public_transaction_limit < 1 {
            apibail_generic!(
                "public transaction limit must be >= 1 in 'network.dht.public_transaction_limit'"
            );
        }
        if self.network.dht.member_transaction_limit < 1 {
            apibail_generic!(
                "member transaction limit must be >= 1 in 'network.dht.member_transaction_limit'"
            );
        }

        Ok(())
    }
}

/// The configuration built for each Veilid node during API startup
#[derive(Clone)]
#[must_use]
pub struct VeilidStartupOptions {
    update_cb: UpdateCallback,
    config: Arc<VeilidConfig>,
}

impl fmt::Debug for VeilidStartupOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VeilidConfig")
            .field("config", self.config.as_ref())
            .finish()
    }
}

impl VeilidStartupOptions {
    pub(crate) fn try_new(
        config: VeilidConfig,
        update_cb: UpdateCallback,
    ) -> VeilidAPIResult<Self> {
        config.validate()?;

        Ok(Self {
            update_cb,
            config: Arc::new(config),
        })
    }

    #[must_use]
    pub fn update_callback(&self) -> UpdateCallback {
        self.update_cb.clone()
    }

    #[must_use]
    pub fn config(&self) -> Arc<VeilidConfig> {
        self.config.clone()
    }
}

/// Return the default veilid config as a json object.
#[must_use]
pub fn default_veilid_config() -> String {
    serialize_json(VeilidConfig::default())
}
