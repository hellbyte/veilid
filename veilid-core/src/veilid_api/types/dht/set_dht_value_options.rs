use crate::{Deserialize, JsonSchema, KeyPair, Serialize};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use crate::Tsify;

#[derive(Debug, JsonSchema, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi)
)]
pub struct AllowOffline(pub bool);
impl Default for AllowOffline {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi)
)]
pub struct SetDHTValueOptions {
    #[schemars(with = "Option<String>")]
    pub writer: Option<KeyPair>,
    /// Defaults to true. If false, the value will not be written if the node is offline,
    /// and a TryAgain error will be returned.
    pub allow_offline: Option<AllowOffline>,
}

impl Default for SetDHTValueOptions {
    fn default() -> Self {
        Self {
            writer: None,
            allow_offline: Some(AllowOffline(true)),
        }
    }
}
