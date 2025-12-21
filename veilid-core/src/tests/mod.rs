#[cfg(all(target_os = "android", feature = "veilid_core_android_tests"))]
mod android;
pub mod common;
pub mod fixtures;
#[cfg(all(target_os = "ios", feature = "veilid_core_ios_tests"))]
mod ios;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod native;

#[allow(unused_imports)]
use super::*;

pub use common::*;

/// Main tests entry point for ios, android, and wasm targets
#[allow(dead_code)]
pub async fn run_all_tests() {
    info!("TEST: test_types");
    crypto::tests::test_types::test_all().await;
    info!("TEST: test_crypto");
    crypto::tests::test_crypto::test_all().await;
    info!("TEST: test_envelope_receipt");
    crypto::tests::test_envelope_receipt::test_all().await;

    info!("TEST: test_veilid_core");
    test_veilid_core::test_all().await;

    info!("TEST: test_table_store");
    table_store::tests::test_table_store::test_all().await;

    info!("TEST: test_protected_store");
    test_protected_store::test_all().await;

    info!("TEST: veilid_api");
    veilid_api::tests::test_all().await;

    info!("TEST: routing_table");
    routing_table::tests::test_all().await;

    info!("TEST: network_manager");
    network_manager::tests::test_all().await;

    info!("TEST: storage_manager");
    storage_manager::tests::test_all().await;

    info!("Finished unit tests");
}
