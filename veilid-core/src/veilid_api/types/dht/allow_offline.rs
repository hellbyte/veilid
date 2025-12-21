use super::*;

#[derive(Debug, JsonSchema, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(into_wasm_abi)
)]
pub struct AllowOffline(pub bool);
impl Default for AllowOffline {
    fn default() -> Self {
        Self(true)
    }
}
