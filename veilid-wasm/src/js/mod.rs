/// Veilid WASM Bindings for Native Javascript / Typescript
pub mod veilid_client_js;
pub mod veilid_crypto_js;
pub mod veilid_routing_context_js;
pub mod veilid_table_db_js;

use super::*;

// API Singleton
lazy_static! {
    static ref VEILID_API: SendWrapper<RefCell<Option<VeilidAPI>>> =
        SendWrapper::new(RefCell::new(None));
    static ref FILTERS: SendWrapper<RefCell<BTreeMap<&'static str, VeilidLayerFilter>>> =
        SendWrapper::new(RefCell::new(BTreeMap::new()));
}

fn get_veilid_api() -> Result<VeilidAPI, veilid_core::VeilidAPIError> {
    (*VEILID_API)
        .borrow()
        .clone()
        .ok_or(VeilidAPIError::NotInitialized)
}

fn take_veilid_api() -> Result<VeilidAPI, VeilidAPIError> {
    (**VEILID_API).take().ok_or(VeilidAPIError::NotInitialized)
}
