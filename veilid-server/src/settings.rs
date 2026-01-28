use clap::ValueEnum;
use directories::*;

use crate::tools::*;
use serde_derive::*;
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;
use veilid_core::tools::*;
use veilid_core::*;

use lazy_static::*;

lazy_static! {
    static ref SYSTEM: sysinfo::System = {
        sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::everything()),
        )
    };
    static ref DISKS: sysinfo::Disks = {
        let mut disks = sysinfo::Disks::new_with_refreshed_list();
        disks.sort_by(|a, b| {
            b.mount_point()
                .to_string_lossy()
                .len()
                .cmp(&a.mount_point().to_string_lossy().len())
        });
        disks
    };
}

pub const PROGRAM_NAME: &str = "veilid-server";

pub fn load_default_config() -> EyreResult<config::Config> {
    #[cfg(not(feature = "geolocation"))]
    let privacy_geolocation_section = "";
    #[cfg(feature = "geolocation")]
    let privacy_geolocation_section = r#"
            country_code_denylist: []
    "#;

    #[cfg(not(feature = "virtual-network"))]
    let virtual_network_section = "";
    #[cfg(feature = "virtual-network")]
    let virtual_network_section = r#"
        virtual_network:
            enabled: false
            server_address: ''
    "#;

    #[cfg(not(feature = "virtual-network"))]
    let virtual_network_server_section = "";
    #[cfg(feature = "virtual-network")]
    let virtual_network_server_section = r#"
    virtual_network_server:
        enabled: false
        tcp:
            listen: true
            listen_address: 'localhost:5149'
        ws:
            listen: true
            listen_address: 'localhost:5148'
    "#;

    #[cfg(not(feature = "enable-protocol-wss"))]
    let protocol_wss_section = "";
    #[cfg(feature = "enable-protocol-wss")]
    let protocol_wss_section = r#"wss:
                connect: true
                listen: false
                max_connections: 256
                listen_address: ':5150'
                path: 'ws'
                # url: ''"#;

    let mut default_config = String::from(
        r#"---
daemon:
    enabled: false
client_api:
    ipc_enabled: true
    ipc_directory: '%IPC_DIRECTORY%'
    network_enabled: false
    listen_address: 'localhost:5959'
auto_attach: true
logging:
    system:
        enabled: false
        level: 'info'
        ignore_log_targets: []
    terminal:
        enabled: true
        level: 'info'
        ignore_log_targets: []
    file:
        enabled: false
        path: ''
        append: true
        level: 'info'
        ignore_log_targets: []
    api:
        enabled: true
        level: 'info'
        ignore_log_targets: []
    otlp:
        enabled: false
        level: 'trace'
        grpc_endpoint: 'localhost:4317'
        ignore_log_targets: []
    flame:
        enabled: false
        path: ''
    perfetto:
        enabled: false
        path: ''
    console:
        enabled: false
testing:
    subnode_index: 0
    subnode_count: 1
%VIRTUAL_NETWORK_SERVER_SECTION%
core:
    capabilities:
        disable: []
    protected_store:
        allow_insecure_fallback: true
        always_use_insecure_storage: true
        directory: '%DIRECTORY%'
        delete: false
        device_encryption_key_password: '%DEVICE_ENCRYPTION_KEY_PASSWORD%'
        new_device_encryption_key_password: %NEW_DEVICE_ENCRYPTION_KEY_PASSWORD%
    table_store:
        directory: '%TABLE_STORE_DIRECTORY%'
        delete: false
    block_store:
        directory: '%BLOCK_STORE_DIRECTORY%'
        delete: false
    network:
        connection_initial_timeout_ms: 2000
        connection_inactivity_timeout_ms: 60000
        max_connections_per_ip4: 32
        max_connections_per_ip6_prefix: 32
        max_connections_per_ip6_prefix_size: 56
        max_connection_frequency_per_min: 128
        client_allowlist_timeout_ms: 300000
        reverse_connection_receipt_time_ms: 5000
        hole_punch_receipt_time_ms: 5000
        network_key_password: null
        disable_capabilites: []
        routing_table:
            public_keys: null
            secret_keys: null
            bootstrap: ['bootstrap-v1.veilid.net']
            bootstrap_keys: ['VLD0:Vj0lKDdUQXmQ5Ol1SZdlvXkBHUccBcQvGLN9vbLSI7k','VLD0:QeQJorqbXtC7v3OlynCZ_W3m76wGNeB5NTF81ypqHAo','VLD0:QNdcl-0OiFfYVj9331XVR6IqZ49NG-E18d5P7lwi4TA']
            limit_over_attached: 64
            limit_fully_attached: 32
            limit_attached_strong: 16
            limit_attached_good: 8
            limit_attached_weak: 4
        rpc:
            concurrency: 0
            queue_size: 1024
            max_timestamp_behind_ms: 10000
            max_timestamp_ahead_ms: 10000
            timeout_ms: 5000
            max_route_hop_count: 4
            default_route_hop_count: 1
        dht:
            max_find_node_count: 20
            resolve_node_timeout_ms: 10000
            resolve_node_count: 1
            resolve_node_fanout: 5
            get_value_timeout_ms: 10000
            get_value_count: 3
            get_value_fanout: 5
            set_value_timeout_ms: 10000
            set_value_count: 5
            set_value_fanout: 5
            consensus_width: 10
            min_peer_count: 20
            min_peer_refresh_time_ms: 60000
            validate_dial_info_receipt_time_ms: 2000
            local_subkey_cache_size: 128
            local_max_subkey_cache_memory_mb: 256
            remote_subkey_cache_size: 1024
            remote_max_records: 65536
            remote_max_subkey_cache_memory_mb: %REMOTE_MAX_SUBKEY_CACHE_MEMORY_MB%
            remote_max_storage_space_mb: 0
            public_watch_limit: 32
            member_watch_limit: 8
            max_watch_expiration_ms: 600000
            public_transaction_limit: 4
            member_transaction_limit: 1
        upnp: false
        detect_address_changes: auto
        restricted_nat_retries: 0
        tls:
            certificate_path: '%CERTIFICATE_PATH%'
            private_key_path: '%PRIVATE_KEY_PATH%'
            connection_initial_timeout_ms: 2000
        protocol:
            udp:
                enabled: true
                socket_pool_size: 0
                listen_address: ':5150'
                # public_address: ''
            tcp:
                connect: true
                listen: true
                max_connections: 256
                listen_address: ':5150'
                #'public_address: ''
            ws:
                connect: true
                listen: true
                max_connections: 256
                listen_address: ':5150'
                path: 'ws'
                # url: 'ws://localhost:5150/ws'
            %PROTOCOL_WSS_SECTION%
        privacy:
            require_inbound_relay: false
        %PRIVACY_GEOLOCATION_SECTION%
        %VIRTUAL_NETWORK_SECTION%
        "#,
    )
    .replace(
        "%IPC_DIRECTORY%",
        &Settings::get_default_ipc_directory().to_string_lossy(),
    )
    .replace(
        "%TABLE_STORE_DIRECTORY%",
        &Settings::get_default_table_store_directory().to_string_lossy(),
    )
    .replace(
        "%BLOCK_STORE_DIRECTORY%",
        &Settings::get_default_block_store_directory().to_string_lossy(),
    )
    .replace(
        "%DIRECTORY%",
        &Settings::get_default_protected_store_directory().to_string_lossy(),
    )
    .replace(
        "%CERTIFICATE_PATH%",
        &Settings::get_default_tls_certificate_path().to_string_lossy(),
    )
    .replace(
        "%PRIVATE_KEY_PATH%",
        &Settings::get_default_tls_private_key_path().to_string_lossy(),
    )
    .replace(
        "%REMOTE_MAX_SUBKEY_CACHE_MEMORY_MB%",
        &Settings::get_default_remote_max_subkey_cache_memory_mb().to_string(),
    )
    .replace("%PRIVACY_GEOLOCATION_SECTION%", privacy_geolocation_section)
    .replace("%VIRTUAL_NETWORK_SECTION%", virtual_network_section)
    .replace(
        "%VIRTUAL_NETWORK_SERVER_SECTION%",
        virtual_network_server_section,
    )
    .replace(
        "%PROTOCOL_WSS_SECTION%",
        protocol_wss_section,
    );

    let dek_password = if let Some(dek_password) = std::env::var_os("DEK_PASSWORD") {
        dek_password
            .to_str()
            .ok_or_else(|| eyre!("DEK_PASSWORD is not valid unicode"))?
            .to_owned()
    } else {
        "".to_owned()
    };
    default_config = default_config.replace("%DEVICE_ENCRYPTION_KEY_PASSWORD%", &dek_password);

    let new_dek_password = if let Some(new_dek_password) = std::env::var_os("NEW_DEK_PASSWORD") {
        format!(
            "'{}'",
            new_dek_password
                .to_str()
                .ok_or_else(|| eyre!("NEW_DEK_PASSWORD is not valid unicode"))?
        )
    } else {
        "null".to_owned()
    };
    default_config =
        default_config.replace("%NEW_DEVICE_ENCRYPTION_KEY_PASSWORD%", &new_dek_password);

    config::Config::builder()
        .add_source(config::File::from_str(
            &default_config,
            config::FileFormat::Yaml,
        ))
        .build()
        .wrap_err("failed to parse default config")
}

pub fn load_config(cfg: config::Config, config_file: &Path) -> EyreResult<config::Config> {
    if let Some(config_file_str) = config_file.to_str() {
        config::Config::builder()
            .add_source(cfg)
            .add_source(config::File::new(config_file_str, config::FileFormat::Yaml))
            .build()
            .wrap_err("failed to load config")
    } else {
        bail!("config file path is not valid UTF-8")
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl<'de> serde::Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_ascii_lowercase().as_str() {
            "off" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid log level: {}",
                s
            ))),
        }
    }
}
impl serde::Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            LogLevel::Off => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };
        s.serialize(serializer)
    }
}

impl From<LogLevel> for veilid_core::VeilidConfigLogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => veilid_core::VeilidConfigLogLevel::Off,
            LogLevel::Error => veilid_core::VeilidConfigLogLevel::Error,
            LogLevel::Warn => veilid_core::VeilidConfigLogLevel::Warn,
            LogLevel::Info => veilid_core::VeilidConfigLogLevel::Info,
            LogLevel::Debug => veilid_core::VeilidConfigLogLevel::Debug,
            LogLevel::Trace => veilid_core::VeilidConfigLogLevel::Trace,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedUrl {
    pub urlstring: String,
    pub url: Url,
}

impl ParsedUrl {
    pub fn offset_port(&mut self, offset: u16) -> EyreResult<()> {
        let new_port = self.url.port().unwrap_or_log() + offset;
        // Bump port on url
        self.url
            .set_port(Some(new_port))
            .map_err(|_| eyre!("failed to set port {new_port} on url {}", self.url.as_str()))?;
        self.urlstring = self.url.to_string();
        Ok(())
    }
    pub fn with_offset_port(&self, offset: u16) -> EyreResult<Self> {
        let mut x = self.clone();
        x.offset_port(offset)?;
        Ok(x)
    }
}

impl FromStr for ParsedUrl {
    type Err = url::ParseError;
    fn from_str(s: &str) -> Result<ParsedUrl, url::ParseError> {
        let mut url = Url::parse(s)?;
        if url.scheme().to_lowercase() == "http" && url.port().is_none() {
            url.set_port(Some(80))
                .map_err(|_| url::ParseError::InvalidPort)?
        }
        if url.scheme().to_lowercase() == "https" && url.port().is_none() {
            url.set_port(Some(443))
                .map_err(|_| url::ParseError::InvalidPort)?;
        }
        let parsed_urlstring = url.to_string();
        Ok(Self {
            urlstring: parsed_urlstring,
            url,
        })
    }
}

impl<'de> serde::Deserialize<'de> for ParsedUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ParsedUrl::from_str(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for ParsedUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.urlstring.serialize(serializer)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NamedSocketAddrs {
    pub name: String,
    pub addrs: Vec<SocketAddr>,
}

impl FromStr for NamedSocketAddrs {
    type Err = std::io::Error;
    fn from_str(s: &str) -> Result<NamedSocketAddrs, std::io::Error> {
        if s.is_empty() {
            return Ok(NamedSocketAddrs {
                name: String::new(),
                addrs: vec![],
            });
        }
        let addr_iter = listen_address_to_socket_addrs(s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        Ok(NamedSocketAddrs {
            name: s.to_owned(),
            addrs: addr_iter,
        })
    }
}

impl<'de> serde::Deserialize<'de> for NamedSocketAddrs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NamedSocketAddrs::from_str(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for NamedSocketAddrs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.name.serialize(serializer)
    }
}

impl NamedSocketAddrs {
    pub fn offset_port(&mut self, offset: u16) -> EyreResult<bool> {
        // Bump port on name
        if let Some(split) = self.name.rfind(':') {
            let hoststr = &self.name[0..split];
            let portstr = &self.name[split + 1..];
            let port: u16 = portstr.parse::<u16>().wrap_err("failed to parse port")? + offset;

            self.name = format!("{}:{}", hoststr, port);
        } else {
            return Ok(false);
        }

        // Bump port on addresses
        for addr in self.addrs.iter_mut() {
            addr.set_port(addr.port() + offset);
        }

        Ok(true)
    }

    pub fn with_offset_port(&self, offset: u16) -> EyreResult<Self> {
        let mut x = self.clone();
        x.offset_port(offset)?;
        Ok(x)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Terminal {
    pub enabled: bool,
    pub level: LogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[cfg(feature = "flame")]
#[derive(Debug, Deserialize, Serialize)]
pub struct Flame {
    pub enabled: bool,
    pub path: String,
}

#[cfg(all(unix, feature = "perfetto"))]
#[derive(Debug, Deserialize, Serialize)]
pub struct Perfetto {
    pub enabled: bool,
    pub path: String,
}

#[cfg(feature = "tokio-console")]
#[derive(Debug, Deserialize, Serialize)]
pub struct Console {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub enabled: bool,
    pub path: String,
    pub append: bool,
    pub level: LogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct System {
    pub enabled: bool,
    pub level: LogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Api {
    pub enabled: bool,
    pub level: LogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[cfg(feature = "opentelemetry-otlp")]
#[derive(Debug, Deserialize, Serialize)]
pub struct Otlp {
    pub enabled: bool,
    pub level: LogLevel,
    pub grpc_endpoint: NamedSocketAddrs,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientApi {
    pub ipc_enabled: bool,
    pub ipc_directory: PathBuf,
    pub network_enabled: bool,
    pub listen_address: NamedSocketAddrs,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Logging {
    pub system: System,
    pub terminal: Terminal,
    pub file: File,
    pub api: Api,
    #[cfg(feature = "opentelemetry-otlp")]
    pub otlp: Otlp,
    #[cfg(feature = "flame")]
    pub flame: Flame,
    #[cfg(all(unix, feature = "perfetto"))]
    pub perfetto: Perfetto,
    #[cfg(feature = "tokio-console")]
    pub console: Console,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Udp {
    pub enabled: bool,
    pub socket_pool_size: u32,
    pub listen_address: NamedSocketAddrs,
    pub public_address: Option<NamedSocketAddrs>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tcp {
    pub connect: bool,
    pub listen: bool,
    pub max_connections: u32,
    pub listen_address: NamedSocketAddrs,
    pub public_address: Option<NamedSocketAddrs>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ws {
    pub connect: bool,
    pub listen: bool,
    pub max_connections: u32,
    pub listen_address: NamedSocketAddrs,
    pub path: PathBuf,
    pub url: Option<ParsedUrl>,
}

#[cfg(feature = "enable-protocol-wss")]
#[derive(Debug, Deserialize, Serialize)]
pub struct Wss {
    pub connect: bool,
    pub listen: bool,
    pub max_connections: u32,
    pub listen_address: NamedSocketAddrs,
    pub path: PathBuf,
    pub url: Option<ParsedUrl>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Protocol {
    pub udp: Udp,
    pub tcp: Tcp,
    pub ws: Ws,
    #[cfg(feature = "enable-protocol-wss")]
    pub wss: Wss,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Privacy {
    pub require_inbound_relay: bool,
    #[cfg(feature = "geolocation")]
    pub country_code_denylist: Vec<CountryCode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tls {
    pub certificate_path: String,
    pub private_key_path: String,
    pub connection_initial_timeout_ms: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rpc {
    pub concurrency: u32,
    pub queue_size: u32,
    pub max_timestamp_behind_ms: Option<u32>,
    pub max_timestamp_ahead_ms: Option<u32>,
    pub timeout_ms: u32,
    pub max_route_hop_count: u8,
    pub default_route_hop_count: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dht {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RoutingTable {
    pub public_keys: Option<veilid_core::PublicKeyGroup>,
    pub secret_keys: Option<veilid_core::SecretKeyGroup>,
    pub bootstrap: Vec<String>,
    pub bootstrap_keys: Vec<veilid_core::PublicKey>,
    pub limit_over_attached: u32,
    pub limit_fully_attached: u32,
    pub limit_attached_strong: u32,
    pub limit_attached_good: u32,
    pub limit_attached_weak: u32,
}

mod auto_bool {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn from_str(s: &str) -> Result<Option<bool>, String> {
        match s {
            "auto" => Ok(None),
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err("Expected 'auto', 'true', or 'false'".to_owned()),
        }
    }

    pub fn serialize<S>(value: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(value) = value {
            value.to_string().serialize(s)
        } else {
            "auto".serialize(s)
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(d)?;
        from_str(s.as_str()).map_err(serde::de::Error::custom)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_test::{assert_tokens, Token};

        #[test]
        fn test_from_str() {
            let s = "auto";
            let b = from_str(s).unwrap();
            assert_eq!(b, None);

            let s = "true";
            let b = from_str(s).unwrap();
            assert_eq!(b, Some(true));

            let s = "false";
            let b = from_str(s).unwrap();
            assert_eq!(b, Some(false));

            let s = "invalid";
            let b = from_str(s).unwrap_err();
            assert_eq!(b, "Expected 'auto', 'true', or 'false'");
        }

        #[test]
        fn test_serde_tokens() {
            #[derive(Debug, Deserialize, Serialize, PartialEq)]
            struct Foo {
                #[serde(with = "super")]
                pub prop: Option<bool>,
            }

            let obj = Foo { prop: Some(true) };
            assert_tokens(
                &obj,
                &[
                    Token::Struct {
                        name: "Foo",
                        len: 1,
                    },
                    Token::Str("prop"),
                    Token::Str("true"),
                    Token::StructEnd,
                ],
            );
            let obj = Foo { prop: Some(false) };
            assert_tokens(
                &obj,
                &[
                    Token::Struct {
                        name: "Foo",
                        len: 1,
                    },
                    Token::Str("prop"),
                    Token::Str("false"),
                    Token::StructEnd,
                ],
            );

            let obj = Foo { prop: None };
            assert_tokens(
                &obj,
                &[
                    Token::Struct {
                        name: "Foo",
                        len: 1,
                    },
                    Token::Str("prop"),
                    Token::Str("auto"),
                    Token::StructEnd,
                ],
            );
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub connection_initial_timeout_ms: u32,
    pub connection_inactivity_timeout_ms: u32,
    pub max_connections_per_ip4: u32,
    pub max_connections_per_ip6_prefix: u32,
    pub max_connections_per_ip6_prefix_size: u32,
    pub max_connection_frequency_per_min: u32,
    pub client_allowlist_timeout_ms: u32,
    pub reverse_connection_receipt_time_ms: u32,
    pub hole_punch_receipt_time_ms: u32,
    pub network_key_password: Option<String>,
    pub routing_table: RoutingTable,
    pub rpc: Rpc,
    pub dht: Dht,
    pub upnp: bool,
    #[serde(with = "auto_bool")]
    pub detect_address_changes: Option<bool>,
    pub restricted_nat_retries: u32,
    pub tls: Tls,
    pub protocol: Protocol,
    pub privacy: Privacy,
    #[cfg(feature = "virtual-network")]
    pub virtual_network: VirtualNetwork,
}

#[cfg(feature = "virtual-network")]
#[derive(Debug, Deserialize, Serialize)]
pub struct VirtualNetwork {
    pub enabled: bool,
    pub server_address: String,
}

#[cfg(feature = "virtual-network")]
#[derive(Debug, Deserialize, Serialize)]
pub struct VirtualNetworkServer {
    pub enabled: bool,
    pub tcp: VirtualNetworkServerTcp,
    pub ws: VirtualNetworkServerWs,
}
#[cfg(feature = "virtual-network")]
#[derive(Debug, Deserialize, Serialize)]
pub struct VirtualNetworkServerTcp {
    pub listen: bool,
    pub listen_address: NamedSocketAddrs,
}
#[cfg(feature = "virtual-network")]
#[derive(Debug, Deserialize, Serialize)]
pub struct VirtualNetworkServerWs {
    pub listen: bool,
    pub listen_address: NamedSocketAddrs,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Testing {
    pub subnode_index: u16,
    pub subnode_count: u16,
    #[cfg(feature = "virtual-network")]
    pub virtual_network_server: VirtualNetworkServer,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TableStore {
    pub directory: String,
    pub delete: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockStore {
    pub directory: String,
    pub delete: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProtectedStore {
    pub allow_insecure_fallback: bool,
    pub always_use_insecure_storage: bool,
    pub directory: String,
    pub delete: bool,
    pub device_encryption_key_password: String,
    pub new_device_encryption_key_password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Capabilities {
    pub disable: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Core {
    pub capabilities: Capabilities,
    pub protected_store: ProtectedStore,
    pub table_store: TableStore,
    pub block_store: BlockStore,
    pub network: Network,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Daemon {
    pub enabled: bool,
    pub pid_file: Option<String>,
    pub chroot: Option<String>,
    pub working_directory: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub stdout_file: Option<String>,
    pub stderr_file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SettingsInner {
    pub daemon: Daemon,
    pub client_api: ClientApi,
    pub auto_attach: bool,
    pub logging: Logging,
    pub testing: Testing,
    pub core: Core,
}

#[derive(Clone, Debug)]
pub struct Settings {
    inner: Arc<RwLock<SettingsInner>>,
}

impl Settings {
    pub fn new(config_file: Option<&OsStr>) -> EyreResult<Self> {
        // Load the default config
        let mut cfg = load_default_config()?;

        // Merge in the config file if we have one
        if let Some(config_file) = config_file {
            let config_file_path = Path::new(config_file);
            // If the user specifies a config file on the command line then it must exist
            cfg = load_config(cfg, config_file_path)?;
        }

        // Generate config
        let mut inner: SettingsInner = cfg.try_deserialize()?;

        // Fill in missing defaults
        if inner.core.network.dht.remote_max_storage_space_mb == 0 {
            inner.core.network.dht.remote_max_storage_space_mb =
                Self::get_default_remote_max_storage_space_mb(&inner);
        }

        //
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn verify(&self) -> EyreResult<()> {
        cfg_if! {
            if #[cfg(windows)] {
                // no ipc setup for windows
            } else {
                let inner = self.inner.read();
                if inner.client_api.ipc_enabled
                    && !Self::get_or_create_private_directory(&inner.client_api.ipc_directory, true)
                {
                    bail!("unable to create default IPC directory {:?}", inner.client_api.ipc_directory);
                }
            }
        }

        Ok(())
    }

    pub fn read(&self) -> RwLockReadGuard<'_, SettingsInner> {
        self.inner.read()
    }
    pub fn write(&self) -> RwLockWriteGuard<'_, SettingsInner> {
        self.inner.write()
    }

    /// Determine default config path
    ///
    /// In a unix-like environment, veilid-server will look for its config file
    /// in /etc/veilid-server. If a config is not found in this location, it will
    /// follow the XDG user directory spec, and look in `~/.config/veilid-server/`.
    ///
    /// For Windows, a user-local config may be created at
    /// `C:\Users\<user>\AppData\Roaming\Veilid\Veilid`, and for macOS, at
    /// `/Users/<user>/Library/Application Support/org.Veilid.Veilid`
    ///
    pub fn get_default_config_path(subpath: &str) -> PathBuf {
        #[cfg(unix)]
        {
            let globalpath = PathBuf::from("/etc/veilid-server");

            if globalpath.exists() {
                return globalpath.join(subpath);
            }
        }

        let mut ts_path = if let Some(my_proj_dirs) = ProjectDirs::from("org", "Veilid", "Veilid") {
            PathBuf::from(my_proj_dirs.config_dir())
        } else {
            PathBuf::from("./")
        };
        ts_path.push(subpath);
        ts_path
    }

    /// Determine default flamegraph output path
    #[cfg(feature = "flame")]
    pub fn get_default_flame_path(subnode_index: u16, subnode_count: u16) -> PathBuf {
        let name = if subnode_count == 1 {
            if subnode_index == 0 {
                "veilid-server.folded".to_owned()
            } else {
                format!("veilid-server-{}.folded", subnode_index)
            }
        } else {
            format!(
                "veilid-server-{}-{}.folded",
                subnode_index,
                subnode_index + subnode_count - 1
            )
        };
        std::env::temp_dir().join(name)
    }

    /// Determine default perfetto output path
    #[cfg(all(unix, feature = "perfetto"))]
    pub fn get_default_perfetto_path(subnode_index: u16, subnode_count: u16) -> PathBuf {
        let name = if subnode_count == 1 {
            if subnode_index == 0 {
                "veilid-server.pftrace".to_owned()
            } else {
                format!("veilid-server-{}.pftrace", subnode_index)
            }
        } else {
            format!(
                "veilid-server-{}-{}.pftrace",
                subnode_index,
                subnode_index + subnode_count - 1
            )
        };
        std::env::temp_dir().join(name)
    }

    #[cfg_attr(windows, expect(dead_code))]
    fn get_or_create_private_directory<P: AsRef<Path>>(path: P, group_read: bool) -> bool {
        let path = path.as_ref();
        if !path.is_dir()
            && (std::fs::create_dir_all(path).is_err()
                || ensure_directory_private_owner(path, group_read).is_err())
        {
            return false;
        }
        true
    }

    fn get_default_directory(subpath: &str) -> PathBuf {
        #[cfg(unix)]
        {
            let globalpath = PathBuf::from("/var/db/veilid-server");

            if globalpath.exists() {
                return globalpath.join(subpath);
            }
        }

        let mut ts_path = if let Some(my_proj_dirs) = ProjectDirs::from("org", "Veilid", "Veilid") {
            PathBuf::from(my_proj_dirs.data_local_dir())
        } else {
            PathBuf::from("./")
        };
        ts_path.push(subpath);
        ts_path
    }

    pub fn get_default_ipc_directory() -> PathBuf {
        cfg_if! {
            if #[cfg(windows)] {
                PathBuf::from(r"\\.\PIPE\veilid-server")
            } else {
                Self::get_default_directory("ipc")
            }
        }
    }

    pub fn get_default_veilid_server_conf_path() -> PathBuf {
        Settings::get_default_config_path("veilid-server.conf")
    }
    pub fn get_default_table_store_directory() -> PathBuf {
        Settings::get_default_directory("table_store")
    }
    pub fn get_default_block_store_directory() -> PathBuf {
        Settings::get_default_directory("block_store")
    }
    pub fn get_default_protected_store_directory() -> PathBuf {
        Settings::get_default_directory("protected_store")
    }
    pub fn get_default_tls_certificate_path() -> PathBuf {
        Settings::get_default_config_path("ssl/certs/server.crt")
    }
    pub fn get_default_tls_private_key_path() -> PathBuf {
        Settings::get_default_config_path("ssl/keys/server.key")
    }

    pub fn get_default_remote_max_subkey_cache_memory_mb() -> u32 {
        if sysinfo::IS_SUPPORTED_SYSTEM {
            ((SYSTEM.free_memory() / (1024u64 * 1024u64)) / 16) as u32
        } else {
            256
        }
    }

    pub fn get_default_remote_max_storage_space_mb(inner: &SettingsInner) -> u32 {
        let dht_storage_path = inner.core.table_store.directory.clone();
        // Sort longer mount point paths first since we want the mount point closest to our table store directory

        if sysinfo::IS_SUPPORTED_SYSTEM {
            for disk in DISKS.list() {
                if dht_storage_path.starts_with(&*disk.mount_point().to_string_lossy()) {
                    let available_mb = disk.available_space() / 1_000_000u64;
                    if available_mb > 40_000 {
                        // Default to 10GB if more than 40GB is available
                        return 10_000;
                    }
                    // Default to 1/4 of the available space, if less than 40GB is available
                    return available_mb as u32;
                }
            }
        }

        // If we can't figure out our storage path go with 1GB of space and pray
        1_000
    }

    pub fn set(&self, key: &str, value: &str) -> EyreResult<()> {
        let mut inner = self.inner.write();

        macro_rules! set_config_value {
            ($innerkey:expr, $value:expr) => {{
                let innerkeyname = &stringify!($innerkey)[6..];
                if innerkeyname == key {
                    match veilid_core::deserialize_json(value) {
                        Ok(v) => {
                            $innerkey = v;
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(eyre!(
                                "invalid type for key {}, value: {}: {}",
                                key,
                                value,
                                e
                            ))
                        }
                    }
                }
            }};
        }

        macro_rules! set_config_value_custom {
            ($innerkey:expr, $value:expr, $deserializer:expr) => {{
                let innerkeyname = &stringify!($innerkey)[6..];
                if innerkeyname == key {
                    match $deserializer(value) {
                        Ok(v) => {
                            $innerkey = v;
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(eyre!(
                                "invalid type for key {}, value: {}: {}",
                                key,
                                value,
                                e
                            ))
                        }
                    }
                }
            }};
        }

        set_config_value!(inner.daemon.enabled, value);
        set_config_value!(inner.daemon.pid_file, value);
        set_config_value!(inner.daemon.chroot, value);
        set_config_value!(inner.daemon.working_directory, value);
        set_config_value!(inner.daemon.user, value);
        set_config_value!(inner.daemon.group, value);
        set_config_value!(inner.daemon.stdout_file, value);
        set_config_value!(inner.daemon.stderr_file, value);

        set_config_value!(inner.client_api.ipc_enabled, value);
        set_config_value!(inner.client_api.ipc_directory, value);
        set_config_value!(inner.client_api.network_enabled, value);
        set_config_value!(inner.client_api.listen_address, value);

        set_config_value!(inner.auto_attach, value);

        set_config_value!(inner.logging.system.enabled, value);
        set_config_value!(inner.logging.system.level, value);
        set_config_value!(inner.logging.system.ignore_log_targets, value);
        set_config_value!(inner.logging.terminal.enabled, value);
        set_config_value!(inner.logging.terminal.level, value);
        set_config_value!(inner.logging.terminal.ignore_log_targets, value);
        set_config_value!(inner.logging.file.enabled, value);
        set_config_value!(inner.logging.file.path, value);
        set_config_value!(inner.logging.file.append, value);
        set_config_value!(inner.logging.file.level, value);
        set_config_value!(inner.logging.file.ignore_log_targets, value);
        set_config_value!(inner.logging.api.enabled, value);
        set_config_value!(inner.logging.api.level, value);
        set_config_value!(inner.logging.api.ignore_log_targets, value);
        #[cfg(feature = "opentelemetry-otlp")]
        {
            set_config_value!(inner.logging.otlp.enabled, value);
            set_config_value!(inner.logging.otlp.level, value);
            set_config_value!(inner.logging.otlp.grpc_endpoint, value);
            set_config_value!(inner.logging.otlp.ignore_log_targets, value);
        }
        #[cfg(feature = "flame")]
        {
            set_config_value!(inner.logging.flame.enabled, value);
            set_config_value!(inner.logging.flame.path, value);
        }
        #[cfg(all(unix, feature = "perfetto"))]
        {
            set_config_value!(inner.logging.perfetto.enabled, value);
            set_config_value!(inner.logging.perfetto.path, value);
        }
        #[cfg(feature = "tokio-console")]
        set_config_value!(inner.logging.console.enabled, value);
        set_config_value!(inner.testing.subnode_index, value);
        #[cfg(feature = "virtual-network")]
        {
            set_config_value!(inner.testing.virtual_network_server.enabled, value);
            set_config_value!(inner.testing.virtual_network_server.tcp.listen, value);
            set_config_value!(
                inner.testing.virtual_network_server.tcp.listen_address,
                value
            );
            set_config_value!(inner.testing.virtual_network_server.ws.listen, value);
            set_config_value!(
                inner.testing.virtual_network_server.ws.listen_address,
                value
            );
        }
        set_config_value!(inner.core.capabilities.disable, value);
        set_config_value!(inner.core.protected_store.allow_insecure_fallback, value);
        set_config_value!(
            inner.core.protected_store.always_use_insecure_storage,
            value
        );
        set_config_value!(inner.core.protected_store.directory, value);
        set_config_value!(inner.core.protected_store.delete, value);
        set_config_value!(
            inner.core.protected_store.device_encryption_key_password,
            value
        );
        set_config_value!(
            inner
                .core
                .protected_store
                .new_device_encryption_key_password,
            value
        );
        set_config_value!(inner.core.table_store.directory, value);
        set_config_value!(inner.core.table_store.delete, value);
        set_config_value!(inner.core.block_store.directory, value);
        set_config_value!(inner.core.block_store.delete, value);
        set_config_value!(inner.core.network.connection_initial_timeout_ms, value);
        set_config_value!(inner.core.network.connection_inactivity_timeout_ms, value);
        set_config_value!(inner.core.network.max_connections_per_ip4, value);
        set_config_value!(inner.core.network.max_connections_per_ip6_prefix, value);
        set_config_value!(
            inner.core.network.max_connections_per_ip6_prefix_size,
            value
        );
        set_config_value!(inner.core.network.max_connection_frequency_per_min, value);
        set_config_value!(inner.core.network.client_allowlist_timeout_ms, value);
        set_config_value!(inner.core.network.reverse_connection_receipt_time_ms, value);
        set_config_value!(inner.core.network.hole_punch_receipt_time_ms, value);
        set_config_value!(inner.core.network.network_key_password, value);
        set_config_value!(inner.core.network.routing_table.public_keys, value);
        set_config_value!(inner.core.network.routing_table.secret_keys, value);
        set_config_value!(inner.core.network.routing_table.bootstrap, value);
        set_config_value!(inner.core.network.routing_table.bootstrap_keys, value);
        set_config_value!(inner.core.network.routing_table.limit_over_attached, value);
        set_config_value!(inner.core.network.routing_table.limit_fully_attached, value);
        set_config_value!(
            inner.core.network.routing_table.limit_attached_strong,
            value
        );
        set_config_value!(inner.core.network.routing_table.limit_attached_good, value);
        set_config_value!(inner.core.network.routing_table.limit_attached_weak, value);
        set_config_value!(inner.core.network.rpc.concurrency, value);
        set_config_value!(inner.core.network.rpc.queue_size, value);
        set_config_value!(inner.core.network.rpc.max_timestamp_behind_ms, value);
        set_config_value!(inner.core.network.rpc.max_timestamp_ahead_ms, value);
        set_config_value!(inner.core.network.rpc.timeout_ms, value);
        set_config_value!(inner.core.network.rpc.max_route_hop_count, value);
        set_config_value!(inner.core.network.rpc.default_route_hop_count, value);
        set_config_value!(inner.core.network.dht.max_find_node_count, value);
        set_config_value!(inner.core.network.dht.resolve_node_timeout_ms, value);
        set_config_value!(inner.core.network.dht.resolve_node_count, value);
        set_config_value!(inner.core.network.dht.resolve_node_fanout, value);
        set_config_value!(inner.core.network.dht.get_value_timeout_ms, value);
        set_config_value!(inner.core.network.dht.get_value_count, value);
        set_config_value!(inner.core.network.dht.get_value_fanout, value);
        set_config_value!(inner.core.network.dht.set_value_timeout_ms, value);
        set_config_value!(inner.core.network.dht.set_value_count, value);
        set_config_value!(inner.core.network.dht.set_value_fanout, value);
        set_config_value!(inner.core.network.dht.consensus_width, value);
        set_config_value!(inner.core.network.dht.min_peer_count, value);
        set_config_value!(inner.core.network.dht.min_peer_refresh_time_ms, value);
        set_config_value!(
            inner.core.network.dht.validate_dial_info_receipt_time_ms,
            value
        );
        set_config_value!(inner.core.network.dht.local_subkey_cache_size, value);
        set_config_value!(
            inner.core.network.dht.local_max_subkey_cache_memory_mb,
            value
        );
        set_config_value!(inner.core.network.dht.remote_subkey_cache_size, value);
        set_config_value!(inner.core.network.dht.remote_max_records, value);
        set_config_value!(
            inner.core.network.dht.remote_max_subkey_cache_memory_mb,
            value
        );
        set_config_value!(inner.core.network.dht.remote_max_storage_space_mb, value);
        set_config_value!(inner.core.network.dht.public_watch_limit, value);
        set_config_value!(inner.core.network.dht.member_watch_limit, value);
        set_config_value!(inner.core.network.dht.max_watch_expiration_ms, value);
        set_config_value!(inner.core.network.dht.public_transaction_limit, value);
        set_config_value!(inner.core.network.dht.member_transaction_limit, value);
        set_config_value!(inner.core.network.upnp, value);
        set_config_value_custom!(
            inner.core.network.detect_address_changes,
            value,
            auto_bool::from_str
        );
        set_config_value!(inner.core.network.restricted_nat_retries, value);
        set_config_value!(inner.core.network.tls.certificate_path, value);
        set_config_value!(inner.core.network.tls.private_key_path, value);
        set_config_value!(inner.core.network.tls.connection_initial_timeout_ms, value);
        set_config_value!(inner.core.network.protocol.udp.enabled, value);
        set_config_value!(inner.core.network.protocol.udp.socket_pool_size, value);
        set_config_value!(inner.core.network.protocol.udp.listen_address, value);
        set_config_value!(inner.core.network.protocol.udp.public_address, value);
        set_config_value!(inner.core.network.protocol.tcp.connect, value);
        set_config_value!(inner.core.network.protocol.tcp.listen, value);
        set_config_value!(inner.core.network.protocol.tcp.max_connections, value);
        set_config_value!(inner.core.network.protocol.tcp.listen_address, value);
        set_config_value!(inner.core.network.protocol.tcp.public_address, value);
        set_config_value!(inner.core.network.protocol.ws.connect, value);
        set_config_value!(inner.core.network.protocol.ws.listen, value);
        set_config_value!(inner.core.network.protocol.ws.max_connections, value);
        set_config_value!(inner.core.network.protocol.ws.listen_address, value);
        set_config_value!(inner.core.network.protocol.ws.path, value);
        set_config_value!(inner.core.network.protocol.ws.url, value);

        cfg_if::cfg_if! {
            if #[cfg(feature="enable-protocol-wss")] {
                set_config_value!(inner.core.network.protocol.wss.connect, value);
                set_config_value!(inner.core.network.protocol.wss.listen, value);
                set_config_value!(inner.core.network.protocol.wss.max_connections, value);
                set_config_value!(inner.core.network.protocol.wss.listen_address, value);
                set_config_value!(inner.core.network.protocol.wss.path, value);
                set_config_value!(inner.core.network.protocol.wss.url, value);
            }
        }
        set_config_value!(inner.core.network.privacy.require_inbound_relay, value);
        #[cfg(feature = "geolocation")]
        set_config_value!(inner.core.network.privacy.country_code_denylist, value);
        #[cfg(feature = "virtual-network")]
        {
            set_config_value!(inner.core.network.virtual_network.enabled, value);
            set_config_value!(inner.core.network.virtual_network.server_address, value);
        }

        Err(eyre!("settings key '{key}' not found"))
    }

    pub fn get_core_config(
        &self,
        subnode: u16,
        subnode_offset: u16,
    ) -> Result<veilid_core::VeilidConfig, VeilidAPIError> {
        let inner = self.inner.clone();

        let inner = inner.read();

        let core_config = VeilidConfig {
            program_name: PROGRAM_NAME.into(),
            namespace: subnode_namespace(subnode),
            capabilities: VeilidConfigCapabilities {
                disable: {
                    let mut caps = Vec::<veilid_core::VeilidCapability>::new();
                    for c in &inner.core.capabilities.disable {
                        let cap = veilid_core::VeilidCapability::from_str(c.as_str())
                            .map_err(VeilidAPIError::generic)?;
                        caps.push(cap);
                    }
                    caps
                },
            },
            protected_store: VeilidConfigProtectedStore {
                allow_insecure_fallback: inner.core.protected_store.allow_insecure_fallback,
                always_use_insecure_storage: inner.core.protected_store.always_use_insecure_storage,
                directory: inner.core.protected_store.directory.clone(),
                delete: inner.core.protected_store.delete,
                device_encryption_key_password: inner
                    .core
                    .protected_store
                    .device_encryption_key_password
                    .clone(),
                new_device_encryption_key_password: inner
                    .core
                    .protected_store
                    .new_device_encryption_key_password
                    .clone(),
            },
            table_store: VeilidConfigTableStore {
                directory: inner.core.table_store.directory.clone(),
                delete: inner.core.table_store.delete,
            },
            block_store: VeilidConfigBlockStore {
                directory: inner.core.block_store.directory.clone(),
                delete: inner.core.block_store.delete,
            },
            network: VeilidConfigNetwork {
                connection_initial_timeout_ms: inner.core.network.connection_initial_timeout_ms,
                connection_inactivity_timeout_ms: inner
                    .core
                    .network
                    .connection_inactivity_timeout_ms,
                max_connections_per_ip4: inner.core.network.max_connections_per_ip4,
                max_connections_per_ip6_prefix: inner.core.network.max_connections_per_ip6_prefix,
                max_connections_per_ip6_prefix_size: inner
                    .core
                    .network
                    .max_connections_per_ip6_prefix_size,
                max_connection_frequency_per_min: inner
                    .core
                    .network
                    .max_connection_frequency_per_min,
                client_allowlist_timeout_ms: inner.core.network.client_allowlist_timeout_ms,
                reverse_connection_receipt_time_ms: inner
                    .core
                    .network
                    .reverse_connection_receipt_time_ms,
                hole_punch_receipt_time_ms: inner.core.network.hole_punch_receipt_time_ms,
                network_key_password: inner.core.network.network_key_password.clone(),
                routing_table: VeilidConfigRoutingTable {
                    public_keys: inner
                        .core
                        .network
                        .routing_table
                        .public_keys
                        .clone()
                        .unwrap_or_default(),
                    secret_keys: inner
                        .core
                        .network
                        .routing_table
                        .secret_keys
                        .clone()
                        .unwrap_or_default(),
                    bootstrap: inner.core.network.routing_table.bootstrap.clone(),
                    bootstrap_keys: inner.core.network.routing_table.bootstrap_keys.clone(),
                    limit_over_attached: inner.core.network.routing_table.limit_over_attached,
                    limit_fully_attached: inner.core.network.routing_table.limit_fully_attached,
                    limit_attached_strong: inner.core.network.routing_table.limit_attached_strong,
                    limit_attached_good: inner.core.network.routing_table.limit_attached_good,
                    limit_attached_weak: inner.core.network.routing_table.limit_attached_weak,
                },
                rpc: VeilidConfigRPC {
                    concurrency: inner.core.network.rpc.concurrency,
                    queue_size: inner.core.network.rpc.queue_size,
                    max_timestamp_behind_ms: inner.core.network.rpc.max_timestamp_behind_ms,
                    max_timestamp_ahead_ms: inner.core.network.rpc.max_timestamp_ahead_ms,
                    timeout_ms: inner.core.network.rpc.timeout_ms,
                    max_route_hop_count: inner.core.network.rpc.max_route_hop_count,
                    default_route_hop_count: inner.core.network.rpc.default_route_hop_count,
                },
                dht: VeilidConfigDHT {
                    max_find_node_count: inner.core.network.dht.max_find_node_count,
                    resolve_node_timeout_ms: inner.core.network.dht.resolve_node_timeout_ms,
                    resolve_node_count: inner.core.network.dht.resolve_node_count,
                    resolve_node_fanout: inner.core.network.dht.resolve_node_fanout,
                    get_value_timeout_ms: inner.core.network.dht.get_value_timeout_ms,
                    get_value_count: inner.core.network.dht.get_value_count,
                    get_value_fanout: inner.core.network.dht.get_value_fanout,
                    set_value_timeout_ms: inner.core.network.dht.set_value_timeout_ms,
                    set_value_count: inner.core.network.dht.set_value_count,
                    set_value_fanout: inner.core.network.dht.set_value_fanout,
                    consensus_width: inner.core.network.dht.consensus_width,
                    min_peer_count: inner.core.network.dht.min_peer_count,
                    min_peer_refresh_time_ms: inner.core.network.dht.min_peer_refresh_time_ms,
                    validate_dial_info_receipt_time_ms: inner
                        .core
                        .network
                        .dht
                        .validate_dial_info_receipt_time_ms,
                    local_subkey_cache_size: inner.core.network.dht.local_subkey_cache_size,
                    local_max_subkey_cache_memory_mb: inner
                        .core
                        .network
                        .dht
                        .local_max_subkey_cache_memory_mb,
                    remote_subkey_cache_size: inner.core.network.dht.remote_subkey_cache_size,
                    remote_max_records: inner.core.network.dht.remote_max_records,
                    remote_max_subkey_cache_memory_mb: inner
                        .core
                        .network
                        .dht
                        .remote_max_subkey_cache_memory_mb,
                    remote_max_storage_space_mb: inner.core.network.dht.remote_max_storage_space_mb,
                    public_watch_limit: inner.core.network.dht.public_watch_limit,
                    member_watch_limit: inner.core.network.dht.member_watch_limit,
                    max_watch_expiration_ms: inner.core.network.dht.max_watch_expiration_ms,
                    public_transaction_limit: inner.core.network.dht.public_transaction_limit,
                    member_transaction_limit: inner.core.network.dht.member_transaction_limit,
                },
                upnp: inner.core.network.upnp,
                detect_address_changes: inner.core.network.detect_address_changes,
                restricted_nat_retries: inner.core.network.restricted_nat_retries,
                tls: VeilidConfigTLS {
                    certificate_path: inner.core.network.tls.certificate_path.clone(),
                    private_key_path: inner.core.network.tls.private_key_path.clone(),
                    connection_initial_timeout_ms: inner
                        .core
                        .network
                        .tls
                        .connection_initial_timeout_ms,
                },
                protocol: VeilidConfigProtocol {
                    udp: VeilidConfigUDP {
                        enabled: inner.core.network.protocol.udp.enabled,
                        socket_pool_size: inner.core.network.protocol.udp.socket_pool_size,
                        listen_address: inner
                            .core
                            .network
                            .protocol
                            .udp
                            .listen_address
                            .with_offset_port(subnode_offset)
                            .map_err(VeilidAPIError::internal)?
                            .name
                            .clone(),
                        public_address: inner
                            .core
                            .network
                            .protocol
                            .udp
                            .public_address
                            .as_ref()
                            .map(|a| a.name.clone()),
                    },
                    tcp: VeilidConfigTCP {
                        connect: inner.core.network.protocol.tcp.connect,
                        listen: inner.core.network.protocol.tcp.listen,
                        max_connections: inner.core.network.protocol.tcp.max_connections,
                        listen_address: inner
                            .core
                            .network
                            .protocol
                            .tcp
                            .listen_address
                            .with_offset_port(subnode_offset)
                            .map_err(VeilidAPIError::internal)?
                            .name
                            .clone(),
                        public_address: inner
                            .core
                            .network
                            .protocol
                            .tcp
                            .public_address
                            .as_ref()
                            .map(|a| a.name.clone()),
                    },
                    ws: VeilidConfigWS {
                        connect: inner.core.network.protocol.ws.connect,
                        listen: inner.core.network.protocol.ws.listen,
                        max_connections: inner.core.network.protocol.ws.max_connections,
                        listen_address: inner
                            .core
                            .network
                            .protocol
                            .ws
                            .listen_address
                            .with_offset_port(subnode_offset)
                            .map_err(VeilidAPIError::internal)?
                            .name
                            .clone(),
                        path: inner
                            .core
                            .network
                            .protocol
                            .ws
                            .path
                            .to_string_lossy()
                            .to_string(),
                        url: match inner.core.network.protocol.ws.url {
                            Some(ref a) => Some(
                                a.with_offset_port(subnode_offset)
                                    .map_err(VeilidAPIError::internal)
                                    .map(|x| x.urlstring.clone())?,
                            ),
                            None => None,
                        },
                    },
                    #[cfg(feature = "enable-protocol-wss")]
                    wss: VeilidConfigWSS {
                        connect: inner.core.network.protocol.wss.connect,
                        listen: inner.core.network.protocol.wss.listen,
                        max_connections: inner.core.network.protocol.wss.max_connections,
                        listen_address: inner
                            .core
                            .network
                            .protocol
                            .wss
                            .listen_address
                            .with_offset_port(subnode_offset)
                            .map_err(VeilidAPIError::internal)?
                            .name
                            .clone(),
                        path: inner
                            .core
                            .network
                            .protocol
                            .wss
                            .path
                            .to_string_lossy()
                            .to_string(),
                        url: match inner.core.network.protocol.wss.url {
                            Some(ref a) => Some(
                                a.with_offset_port(subnode_offset)
                                    .map_err(VeilidAPIError::internal)
                                    .map(|x| x.urlstring.clone())?,
                            ),
                            None => None,
                        },
                    },
                },
                privacy: VeilidConfigPrivacy {
                    require_inbound_relay: inner.core.network.privacy.require_inbound_relay,
                    #[cfg(feature = "geolocation")]
                    country_code_denylist: inner.core.network.privacy.country_code_denylist.clone(),
                },
                #[cfg(feature = "virtual-network")]
                virtual_network: VeilidConfigVirtualNetwork {
                    enabled: inner.core.network.virtual_network.enabled,
                    server_address: inner.core.network.virtual_network.server_address.clone(),
                },
            },
        };

        Ok(core_config)
    }
}

pub fn subnode_namespace(subnode_index: u16) -> String {
    if subnode_index == 0 {
        "".to_owned()
    } else {
        format!("subnode{}", subnode_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = load_default_config().unwrap_or_log();
        let inner = cfg.try_deserialize::<SettingsInner>().unwrap_or_log();
        println!("default settings: {:?}", inner);
    }

    #[test]
    fn test_default_config_settings() {
        let settings = Settings::new(None).unwrap_or_log();

        let s = settings.read();
        assert!(!s.daemon.enabled);
        assert_eq!(s.daemon.pid_file, None);
        assert_eq!(s.daemon.chroot, None);
        assert_eq!(s.daemon.working_directory, None);
        assert_eq!(s.daemon.user, None);
        assert_eq!(s.daemon.group, None);
        assert_eq!(s.daemon.stdout_file, None);
        assert_eq!(s.daemon.stderr_file, None);
        assert!(s.client_api.ipc_enabled);
        assert!(!s.client_api.network_enabled);
        assert_eq!(s.client_api.listen_address.name, "localhost:5959");
        assert_eq!(
            s.client_api.listen_address.addrs,
            listen_address_to_socket_addrs("localhost:5959").unwrap_or_log()
        );
        assert!(s.auto_attach);
        assert!(!s.logging.system.enabled);
        assert_eq!(s.logging.system.level, LogLevel::Info);
        assert!(s.logging.terminal.enabled);
        assert_eq!(s.logging.terminal.level, LogLevel::Info);
        assert!(!s.logging.file.enabled);
        assert_eq!(s.logging.file.path, "");
        assert!(s.logging.file.append);
        assert_eq!(s.logging.file.level, LogLevel::Info);
        assert!(s.logging.api.enabled);
        assert_eq!(s.logging.api.level, LogLevel::Info);
        assert!(!s.logging.otlp.enabled);
        assert_eq!(s.logging.otlp.level, LogLevel::Trace);
        assert_eq!(
            s.logging.otlp.grpc_endpoint,
            NamedSocketAddrs::from_str("localhost:4317").unwrap_or_log()
        );
        #[cfg(feature = "flame")]
        {
            assert!(!s.logging.flame.enabled);
            assert_eq!(s.logging.flame.path, "");
        }
        #[cfg(all(unix, feature = "perfetto"))]
        {
            assert!(!s.logging.perfetto.enabled);
            assert_eq!(s.logging.perfetto.path, "");
        }
        #[cfg(feature = "tokio-console")]
        assert!(!s.logging.console.enabled);
        assert_eq!(s.testing.subnode_index, 0);
        #[cfg(feature = "virtual-network")]
        {
            assert_eq!(s.testing.virtual_network_server.enabled, false);
            assert_eq!(s.testing.virtual_network_server.tcp.listen, false);
            assert_eq!(
                s.testing.virtual_network_server.tcp.listen_address,
                "localhost:5149"
            );
            assert_eq!(s.testing.virtual_network_server.ws.listen, false);
            assert_eq!(
                s.testing.virtual_network_server.ws.listen_address,
                "localhost:5148"
            );
        }
        assert_eq!(
            s.core.table_store.directory,
            Settings::get_default_table_store_directory()
                .to_string_lossy()
                .to_string()
        );
        assert!(!s.core.table_store.delete);

        assert_eq!(
            s.core.block_store.directory,
            Settings::get_default_block_store_directory()
                .to_string_lossy()
                .to_string()
        );
        assert!(!s.core.block_store.delete);

        assert!(s.core.protected_store.allow_insecure_fallback);
        assert!(s.core.protected_store.always_use_insecure_storage);
        assert_eq!(
            s.core.protected_store.directory,
            Settings::get_default_protected_store_directory()
                .to_string_lossy()
                .to_string()
        );
        assert!(!s.core.protected_store.delete);
        assert_eq!(s.core.protected_store.device_encryption_key_password, "");
        assert_eq!(
            s.core.protected_store.new_device_encryption_key_password,
            None
        );

        assert_eq!(s.core.network.connection_initial_timeout_ms, 2_000u32);
        assert_eq!(s.core.network.connection_inactivity_timeout_ms, 60_000u32);
        assert_eq!(s.core.network.max_connections_per_ip4, 32u32);
        assert_eq!(s.core.network.max_connections_per_ip6_prefix, 32u32);
        assert_eq!(s.core.network.max_connections_per_ip6_prefix_size, 56u32);
        assert_eq!(s.core.network.max_connection_frequency_per_min, 128u32);
        assert_eq!(s.core.network.client_allowlist_timeout_ms, 300_000u32);
        assert_eq!(s.core.network.reverse_connection_receipt_time_ms, 5_000u32);
        assert_eq!(s.core.network.hole_punch_receipt_time_ms, 5_000u32);
        assert_eq!(s.core.network.network_key_password, None);
        assert_eq!(s.core.network.routing_table.public_keys, None);
        assert_eq!(s.core.network.routing_table.secret_keys, None);
        //
        assert_eq!(
            s.core.network.routing_table.bootstrap,
            vec!["bootstrap-v1.veilid.net".to_owned()]
        );
        assert_eq!(
            s.core.network.routing_table.bootstrap_keys,
            vec![
                PublicKey::from_str("VLD0:Vj0lKDdUQXmQ5Ol1SZdlvXkBHUccBcQvGLN9vbLSI7k")
                    .unwrap_or_log(),
                PublicKey::from_str("VLD0:QeQJorqbXtC7v3OlynCZ_W3m76wGNeB5NTF81ypqHAo")
                    .unwrap_or_log(),
                PublicKey::from_str("VLD0:QNdcl-0OiFfYVj9331XVR6IqZ49NG-E18d5P7lwi4TA")
                    .unwrap_or_log(),
            ]
        );
        //
        assert_eq!(s.core.network.rpc.concurrency, 0);
        assert_eq!(s.core.network.rpc.queue_size, 1024);
        assert_eq!(s.core.network.rpc.max_timestamp_behind_ms, Some(10_000u32));
        assert_eq!(s.core.network.rpc.max_timestamp_ahead_ms, Some(10_000u32));
        assert_eq!(s.core.network.rpc.timeout_ms, 5_000u32);
        assert_eq!(s.core.network.rpc.max_route_hop_count, 4);
        assert_eq!(s.core.network.rpc.default_route_hop_count, 1);
        //
        assert_eq!(s.core.network.dht.max_find_node_count, 20u32);
        assert_eq!(s.core.network.dht.resolve_node_timeout_ms, 10_000u32);
        assert_eq!(s.core.network.dht.resolve_node_count, 1u32);
        assert_eq!(s.core.network.dht.resolve_node_fanout, 5u32);
        assert_eq!(s.core.network.dht.get_value_timeout_ms, 10_000u32);
        assert_eq!(s.core.network.dht.get_value_count, 3u32);
        assert_eq!(s.core.network.dht.get_value_fanout, 5u32);
        assert_eq!(s.core.network.dht.set_value_timeout_ms, 10_000u32);
        assert_eq!(s.core.network.dht.set_value_count, 5u32);
        assert_eq!(s.core.network.dht.set_value_fanout, 5u32);
        assert_eq!(s.core.network.dht.consensus_width, 10u32);
        assert_eq!(s.core.network.dht.min_peer_count, 20u32);
        assert_eq!(s.core.network.dht.min_peer_refresh_time_ms, 60_000u32);
        assert_eq!(
            s.core.network.dht.validate_dial_info_receipt_time_ms,
            2_000u32
        );
        assert_eq!(s.core.network.dht.public_watch_limit, 32u32);
        assert_eq!(s.core.network.dht.member_watch_limit, 8u32);
        assert_eq!(s.core.network.dht.max_watch_expiration_ms, 600_000u32);
        assert_eq!(s.core.network.dht.public_transaction_limit, 4u32);
        assert_eq!(s.core.network.dht.member_transaction_limit, 1u32);
        //
        assert!(!s.core.network.upnp);
        assert_eq!(s.core.network.detect_address_changes, None);
        assert_eq!(s.core.network.restricted_nat_retries, 0u32);
        //
        assert_eq!(
            s.core.network.tls.certificate_path,
            Settings::get_default_tls_certificate_path()
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(
            s.core.network.tls.private_key_path,
            Settings::get_default_tls_private_key_path()
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(s.core.network.tls.connection_initial_timeout_ms, 2_000u32);
        //
        //
        let valid_socket_addrs = [
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 5150),
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 5150),
        ];

        assert!(s.core.network.protocol.udp.enabled);
        assert_eq!(s.core.network.protocol.udp.socket_pool_size, 0);
        assert_eq!(s.core.network.protocol.udp.listen_address.name, ":5150");
        for addr in &s.core.network.protocol.udp.listen_address.addrs {
            assert!(valid_socket_addrs.contains(addr));
        }
        assert!(!s.core.network.protocol.udp.listen_address.addrs.is_empty());
        assert_eq!(s.core.network.protocol.udp.public_address, None);

        //
        assert!(s.core.network.protocol.tcp.connect);
        assert!(s.core.network.protocol.tcp.listen);
        assert_eq!(s.core.network.protocol.tcp.max_connections, 256);
        assert_eq!(s.core.network.protocol.tcp.listen_address.name, ":5150");
        for addr in &s.core.network.protocol.tcp.listen_address.addrs {
            assert!(valid_socket_addrs.contains(addr));
        }
        assert!(!s.core.network.protocol.tcp.listen_address.addrs.is_empty());
        assert_eq!(s.core.network.protocol.tcp.public_address, None);

        //
        assert!(s.core.network.protocol.ws.connect);
        assert!(s.core.network.protocol.ws.listen);
        assert_eq!(s.core.network.protocol.ws.max_connections, 256);
        assert_eq!(s.core.network.protocol.ws.listen_address.name, ":5150");
        for addr in &s.core.network.protocol.ws.listen_address.addrs {
            assert!(valid_socket_addrs.contains(addr));
        }
        assert!(!s.core.network.protocol.ws.listen_address.addrs.is_empty());
        assert_eq!(
            s.core.network.protocol.ws.path,
            std::path::PathBuf::from("ws")
        );
        assert_eq!(s.core.network.protocol.ws.url, None);
        //
        cfg_if::cfg_if! {
            if #[cfg(feature="enable-protocol-wss")] {
                assert!(s.core.network.protocol.wss.connect);
                assert!(!s.core.network.protocol.wss.listen);
                assert_eq!(s.core.network.protocol.wss.max_connections, 256);
                assert_eq!(s.core.network.protocol.wss.listen_address.name, ":5150");
                for addr in &s.core.network.protocol.wss.listen_address.addrs {
                    assert!(valid_socket_addrs.contains(addr));
                }
                assert!(!s.core.network.protocol.wss.listen_address.addrs.is_empty());
                assert_eq!(
                    s.core.network.protocol.wss.path,
                    std::path::PathBuf::from("ws")
                );
                assert_eq!(s.core.network.protocol.wss.url, None);
            }
        }
        //
        assert!(!s.core.network.privacy.require_inbound_relay);
        #[cfg(feature = "geolocation")]
        assert_eq!(s.core.network.privacy.country_code_denylist, &[]);
        #[cfg(feature = "virtual-network")]
        {
            assert_eq!(s.core.network.virtual_network.enabled, false);
            assert_eq!(s.core.network.virtual_network.server_address, "");
        }
    }
}
