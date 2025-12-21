use super::*;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
pub struct RouteBlob {
    #[serde(with = "as_human_string")]
    #[schemars(with = "String")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        tsify(type = "string")
    )]
    pub route_id: RouteId,
    #[cfg_attr(
        not(all(target_arch = "wasm32", target_os = "unknown")),
        serde(with = "as_human_base64")
    )]
    #[schemars(with = "String")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        serde(with = "serde_bytes"),
        tsify(type = "Uint8Array")
    )]
    pub blob: Vec<u8>,
}
