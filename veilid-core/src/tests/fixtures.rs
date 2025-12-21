use crate::*;

cfg_if! {
    if #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))] {
        use std::fs::File;
        use std::io::prelude::*;
        use std::path::PathBuf;

        static CERTFILE: &str = r#"-----BEGIN CERTIFICATE-----
MIIDbzCCAlegAwIBAgIRALB/PvRpqN55Pk7L33NNsvcwDQYJKoZIhvcNAQELBQAw
FDESMBAGA1UEAwwJTm9jdGVtIENBMB4XDTIwMDkwODIxMDkwMFoXDTMwMDkwNjIx
MDkwMFowHDEaMBgGA1UEAwwRKi5ub2N0ZW0uaW50ZXJuYWwwggEiMA0GCSqGSIb3
DQEBAQUAA4IBDwAwggEKAoIBAQDRbAtA2dIlTPaQUN43/bdGi2wuDzCXk36TcfOr
YoxGsyJV6QpcIdmtrPN2WbkuDmA/G+0BUcQPvBfA/pFRHQElrzMhGR23Mp6IK7YR
pomUa1DQSJyMw/WM9V0+tidp5tJSeUCB+qKhLBrztD5XXjdhU6WA1J0y26XQoBqs
RZbPV8mce4LxVaQptkf4NB4/jnr3M1/FWEri60xBw3blWGaLP6gza3vqAr8pqEY4
zXU4q+egLbRIOwxwBJ0/vcyO6BdSzA1asWJCddXQJkUQrLl3OQ+44FMsAFyzCOiK
DVoqD2z4IJvIRT6TH8OcYvrotytlsNXS4ja9r32tTR1/DxUrAgMBAAGjgbMwgbAw
CQYDVR0TBAIwADAdBgNVHQ4EFgQUhjP4CArB3wWGHfavf7mRxaYshKMwRAYDVR0j
BD0wO4AUKAOv10AaiIUHgOtx0Mk6ZaZ/tGWhGKQWMBQxEjAQBgNVBAMMCU5vY3Rl
bSBDQYIJAISVWafozd3RMBMGA1UdJQQMMAoGCCsGAQUFBwMBMAsGA1UdDwQEAwIF
oDAcBgNVHREEFTATghEqLm5vY3RlbS5pbnRlcm5hbDANBgkqhkiG9w0BAQsFAAOC
AQEAMfVGtpXdkxflSQY2DzIUXLp9cZQnu4A8gww8iaLAg5CIUijP71tb2JJ+SsRx
W3p14YMhOYtswIvGTtXWzMgfAivwrxCcJefnqDAG9yviWoA0CSQe21nRjEqN6nyh
CS2BIkOcNNf10TD9sNo7z6IIXNjok7/F031JvH6pBgZ8Bq4IE/ANIuAvxwslPrqT
80qnWtAc5TzNNR1CT+fyZwMEpeW5fMZQnrSyUMsNv06Jydl/7IkGvlmbwihZOg95
Vty37pyzrXU5s/DY1zi5aYoFiK7/4bNEy9mRL9ero+kCvQfea0Yt2rITKQkCYvKu
MQTNaSyo6GTifW5InckkQIsnTQ==
-----END CERTIFICATE-----"#;

        static KEYFILE: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDRbAtA2dIlTPaQ
UN43/bdGi2wuDzCXk36TcfOrYoxGsyJV6QpcIdmtrPN2WbkuDmA/G+0BUcQPvBfA
/pFRHQElrzMhGR23Mp6IK7YRpomUa1DQSJyMw/WM9V0+tidp5tJSeUCB+qKhLBrz
tD5XXjdhU6WA1J0y26XQoBqsRZbPV8mce4LxVaQptkf4NB4/jnr3M1/FWEri60xB
w3blWGaLP6gza3vqAr8pqEY4zXU4q+egLbRIOwxwBJ0/vcyO6BdSzA1asWJCddXQ
JkUQrLl3OQ+44FMsAFyzCOiKDVoqD2z4IJvIRT6TH8OcYvrotytlsNXS4ja9r32t
TR1/DxUrAgMBAAECggEBAMIAK+CUqCbjyBliwKjvwWN5buqwKZyRBxXB3y/qJ/aq
pWkea/lzZjqMWDFP5sryiFiOHx00yMKmxP6FFMsmalSlm2DS6oM2QkP08kIhm5vB
WmjIizWfpo5BEnMwvQxOxpGeP5LpQtS5jfIrDAFVh0oC+fOBgmqFrXK5jlv+Tzmc
9PzoF5lgy8CHw3NxuScJpEhA1vTzu5N7sTdiTDKqY1ph2+RFlf30oyx4whoRVpIC
w8vp3WbLu/yAGuN5S14mYJW2Qgi8/rVCDStROEKOeB99mt1MG5lX7iuagzS/95Lr
2m1Nya0+7hkkpq6Y3Wqne9H0NLasJK8PU8ZaEc6BwTkCgYEA8iLVBrt4W/Cc5hry
8LWCMX8P25z7WIRYswnPvqwTwE0f6Q1ddWIaR9GPWUHgoRC4Z0b0MKolwo9s8RPE
GBuTOCy8ArSgYb1jNpsanGIWg6mZZgfylKdMdCMXMAAYF1/sTXeqCDY+FSCzEAvZ
hzppcCpiKV7Pa9aOo7o3/IeUBZcCgYEA3WmyvscG27R18XASJYL8Y4DuFvvnTHMp
YnxJIoS1+0TnUD2QqXUnXKbnTioWs7t990YAjbsHvK4fVsbnkuEm/as0oYbC8vU1
W3XN0HrpiacGcYIcXU4AY4XvY8t3y76FycJAT9Q6QztVofI5DmXV+8qsyrEegUys
wPIkkumCJ40CgYBKT3hTPZudk8WDNQgT6ZCQQi+Kta3Jp6xVHhC8srDJFqJRcsGY
8ceg/OZifT5EEA6X24W7naxC/qNvhSJsR6Ix3kDBD9AczvOw4X8UOWIxfA5Q6uV+
y61CAzbti0nZep3Z1HzBUmxRLZzmssxKnRmYy9keWzOLI+jYxKDEBpPd9wKBgAY1
pquvDUQwJXal+/xNViK8RPEkE3KTcD+w2KQ9MJVhc1NOxrXZ8Uap76bDi2tzAK9k
qTNQYYErKPnYDjqSUfOfT5SQIPuLYPm1rhYAvHf91TJtwbnkLCKeaP5VgICYUUw9
RGx4uUGVcmteTbdXp86t+naczQw3SEkJAXmVTu8pAoGATF7xXifMUSL1v43Ybrmc
RikQyDecRspMYLOCNmPWI2PPz6MAjm8jDCsXK52HUK4mUqrd/W3rqnl+TrJsXOnH
Ww6tESPaF1kCVyV2Jx/5m8qsE9y5Bds7eMo2JF8vnAKFX6t4KwZiyHBymj6uelNc
wFAbkZY9eS/x6P7qrpd7dUA=
-----END PRIVATE KEY-----"#;
    }
}

cfg_if! {

    if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
        #[must_use]pub fn get_table_store_path() -> String {
            String::new()
        }
        #[must_use]pub fn get_block_store_path() -> String {
            String::new()
        }
        #[must_use]pub fn get_protected_store_path() -> String {
            String::new()
        }
        #[must_use]pub fn get_certfile_path() -> String {
            String::new()
        }
        #[must_use]pub fn get_keyfile_path() -> String {
            String::new()
        }
    }
    else {

        #[must_use] fn get_data_dir() -> PathBuf {
            cfg_if! {
                if #[cfg(target_os = "android")] {
                    PathBuf::from(crate::intf::android::get_files_dir())
                } else {
                    use directories::*;

                    if let Some(my_proj_dirs) = ProjectDirs::from("org", "Veilid", "VeilidCoreTests") {
                        PathBuf::from(my_proj_dirs.data_local_dir())
                    } else {
                        PathBuf::from("./")
                    }
                }
            }
        }

        #[must_use] pub fn get_table_store_path() -> String {
            let mut out = get_data_dir();
            std::fs::create_dir_all(&out).unwrap();

            out.push("table_store");

            out.into_os_string().into_string().unwrap()
        }

        #[must_use] pub fn get_block_store_path() -> String {
            let mut out = get_data_dir();
            std::fs::create_dir_all(&out).unwrap();

            out.push("block_store");

            out.into_os_string().into_string().unwrap()
        }

        #[must_use]        pub fn get_protected_store_path() -> String {
            let mut out = get_data_dir();
            std::fs::create_dir_all(&out).unwrap();

            out.push("protected_store");

            out.into_os_string().into_string().unwrap()
        }

        #[must_use]pub fn get_certfile_path() -> String {
            let mut out = get_data_dir();
            std::fs::create_dir_all(&out).unwrap();

            out.push("cert.pem");
            // Initialize certfile
            if !out.exists() {
                debug!("creating certfile at {:?}", out);
                File::create(&out).unwrap().write_all(CERTFILE.as_bytes()).unwrap();
            }

            out.into_os_string().into_string().unwrap()
        }

        #[must_use]pub fn get_keyfile_path() -> String {
            let mut out = get_data_dir();
            std::fs::create_dir_all(&out).unwrap();

            out.push("key.pem");

            // Initialize keyfile
            if !out.exists() {
                debug!("creating keyfile at {:?}", out);
                File::create(&out).unwrap().write_all(KEYFILE.as_bytes()).unwrap();
            }

            out.into_os_string().into_string().unwrap()
        }
    }
}

fn update_callback(update: VeilidUpdate) {
    info!("update_callback: {:?}", update);
}

pub fn setup_veilid_core() -> (UpdateCallback, VeilidConfig) {
    (Arc::new(update_callback), get_config())
}

pub fn setup_veilid_core_with_namespace<S: AsRef<str>>(
    namespace: S,
) -> (UpdateCallback, VeilidConfig) {
    let namespace = namespace.as_ref().to_string();

    let config = get_config();

    (
        Arc::new(update_callback),
        VeilidConfig {
            namespace: namespace.clone(),
            ..config
        },
    )
}

pub fn fix_fake_veilid_config() -> VeilidConfig {
    VeilidConfig {
        program_name: "Bob".to_string(),
        namespace: "Internets".to_string(),
        capabilities: VeilidConfigCapabilities {
            disable: Vec::new(),
        },
        protected_store: VeilidConfigProtectedStore {
            allow_insecure_fallback: true,
            always_use_insecure_storage: false,
            directory: "/root".to_string(),
            delete: true,
            device_encryption_key_password: "1234".to_string(),
            new_device_encryption_key_password: Some("5678".to_string()),
        },
        table_store: VeilidConfigTableStore {
            directory: "Yellow Pages".to_string(),
            delete: false,
        },
        block_store: VeilidConfigBlockStore {
            directory: "C:\\Program Files".to_string(),
            delete: true,
        },
        network: VeilidConfigNetwork {
            connection_initial_timeout_ms: 1000,
            connection_inactivity_timeout_ms: 2000,
            max_connections_per_ip4: 3000,
            max_connections_per_ip6_prefix: 4000,
            max_connections_per_ip6_prefix_size: 5000,
            max_connection_frequency_per_min: 6000,
            client_allowlist_timeout_ms: 7000,
            reverse_connection_receipt_time_ms: 8000,
            hole_punch_receipt_time_ms: 9000,
            network_key_password: None,
            routing_table: VeilidConfigRoutingTable {
                public_keys: PublicKeyGroup::new(),
                secret_keys: SecretKeyGroup::new(),
                bootstrap: vec!["boots".to_string()],
                bootstrap_keys: vec![PublicKey::from_str(
                    "VLD0:qrxwD1-aM9xiUw4IAPVXE_4qgoIfyR4Y6MEPyaDl_GQ",
                )
                .unwrap()],
                limit_over_attached: 1,
                limit_fully_attached: 2,
                limit_attached_strong: 3,
                limit_attached_good: 4,
                limit_attached_weak: 5,
            },
            rpc: VeilidConfigRPC {
                concurrency: 5,
                queue_size: 6,
                max_timestamp_behind_ms: Some(1000),
                max_timestamp_ahead_ms: Some(2000),
                timeout_ms: 3000,
                max_route_hop_count: 7,
                default_route_hop_count: 8,
            },
            dht: VeilidConfigDHT {
                max_find_node_count: 1,
                resolve_node_timeout_ms: 10000,
                resolve_node_count: 3,
                resolve_node_fanout: 4,
                get_value_timeout_ms: 100000,
                get_value_count: 3,
                get_value_fanout: 4,
                set_value_timeout_ms: 10000,
                set_value_count: 5,
                set_value_fanout: 4,
                consensus_width: 10,
                min_peer_count: 11,
                min_peer_refresh_time_ms: 12,
                validate_dial_info_receipt_time_ms: 13,
                local_subkey_cache_size: 14,
                local_max_subkey_cache_memory_mb: 15,
                remote_subkey_cache_size: 16,
                remote_max_records: 17,
                remote_max_subkey_cache_memory_mb: 18,
                remote_max_storage_space_mb: 19,
                public_watch_limit: 20,
                member_watch_limit: 21,
                max_watch_expiration_ms: 22,
                public_transaction_limit: 23,
                member_transaction_limit: 24,
            },
            upnp: true,
            detect_address_changes: Some(false),
            restricted_nat_retries: 10000,
            tls: VeilidConfigTLS {
                certificate_path: "/etc/ssl/certs/cert.pem".to_string(),
                private_key_path: "/etc/ssl/keys/key.pem".to_string(),
                connection_initial_timeout_ms: 1000,
            },
            protocol: VeilidConfigProtocol {
                udp: VeilidConfigUDP {
                    enabled: false,
                    socket_pool_size: 30,
                    listen_address: "10.0.0.2".to_string(),
                    public_address: Some("2.3.4.5".to_string()),
                },
                tcp: VeilidConfigTCP {
                    connect: true,
                    listen: false,
                    max_connections: 8,
                    listen_address: "10.0.0.1".to_string(),
                    public_address: Some("1.2.3.4".to_string()),
                },
                ws: VeilidConfigWS {
                    connect: false,
                    listen: true,
                    max_connections: 9,
                    listen_address: "127.0.0.1".to_string(),
                    path: "Straight".to_string(),
                    url: Some("https://veilid.com/ws".to_string()),
                },
                #[cfg(feature = "enable-protocol-wss")]
                wss: VeilidConfigWSS {
                    connect: true,
                    listen: false,
                    max_connections: 10,
                    listen_address: "::1".to_string(),
                    path: "Curved".to_string(),
                    url: Some("https://veilid.com/wss".to_string()),
                },
            },
            privacy: VeilidConfigPrivacy {
                require_inbound_relay: false,
                #[cfg(feature = "geolocation")]
                country_code_denylist: vec![CountryCode::from_str("NZ").unwrap()],
            },
            #[cfg(feature = "virtual-network")]
            virtual_network: VeilidConfigVirtualNetwork {
                enabled: false,
                server_address: "".to_owned(),
            },
        },
    }
}

pub fn get_startup_options() -> VeilidStartupOptions {
    VeilidStartupOptions::try_new(get_config(), Arc::new(update_callback)).unwrap()
}

pub fn get_config() -> VeilidConfig {
    VeilidConfig {
        program_name: "VeilidCoreTests".into(),
        table_store: VeilidConfigTableStore {
            directory: get_table_store_path(),
            delete: true,
        },
        block_store: VeilidConfigBlockStore {
            directory: get_block_store_path(),
            delete: true,
        },
        protected_store: VeilidConfigProtectedStore {
            allow_insecure_fallback: true,
            always_use_insecure_storage: true,
            directory: get_protected_store_path(),
            delete: true,
            ..Default::default()
        },
        network: VeilidConfigNetwork {
            upnp: false,
            tls: VeilidConfigTLS {
                certificate_path: get_certfile_path(),
                private_key_path: get_keyfile_path(),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

// Network-related code to make sure veilid node is connetected to other peers
pub async fn wait_for_public_internet_ready(api: &VeilidAPI) {
    info!("wait_for_public_internet_ready");
    loop {
        let state = api.get_state().await.unwrap();
        if state.attachment.public_internet_ready {
            break;
        }
        sleep(1000).await;
    }
    info!("wait_for_public_internet_ready, done");
}
