mod test_crypto;
mod test_envelope_receipt;
mod test_types;

pub mod mocks;
pub use mocks::*;

use super::*;
use crate::tests::*;

async fn crypto_tests_startup() -> VeilidAPI {
    trace!("crypto_tests: starting");
    let (update_callback, config) = fixture_veilid_core();

    api_startup(update_callback, config)
        .await
        .expect("startup failed")
}

async fn crypto_tests_shutdown(api: VeilidAPI) {
    trace!("crypto_tests: shutting down");
    api.shutdown().await;
    trace!("crypto_tests: finished");
}

pub async fn test_all() {
    test_types::test_all().await;
    test_crypto::test_all().await;
    test_envelope_receipt::test_all().await;
}
