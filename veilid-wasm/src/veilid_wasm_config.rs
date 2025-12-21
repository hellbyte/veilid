use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi)
)]
pub enum VeilidWASMConfigLoggingLogsInConsole {
    Off,
    NoColor,
    Color,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi),
    serde(rename_all = "camelCase")
)]
pub struct VeilidWASMConfigLoggingPerformance {
    pub enabled: bool,
    pub level: veilid_core::VeilidConfigLogLevel,
    pub logs_in_timings: bool,
    pub logs_in_console: VeilidWASMConfigLoggingLogsInConsole,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi),
    serde(rename_all = "camelCase")
)]
pub struct VeilidWASMConfigLoggingAPI {
    pub enabled: bool,
    pub level: veilid_core::VeilidConfigLogLevel,
    pub ignore_log_targets: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi),
    serde(rename_all = "camelCase")
)]
pub struct VeilidWASMConfigLogging {
    pub performance: VeilidWASMConfigLoggingPerformance,
    pub api: VeilidWASMConfigLoggingAPI,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi),
    serde(rename_all = "camelCase")
)]
pub struct VeilidWASMConfig {
    pub logging: VeilidWASMConfigLogging,
}
