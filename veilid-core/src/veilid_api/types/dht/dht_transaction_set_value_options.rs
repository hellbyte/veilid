use super::*;

/// Options that override defaults for DHTTransaction::set
#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
pub struct DHTTransactionSetValueOptions {
    #[schemars(with = "Option<String>")]
    pub writer: Option<KeyPair>,
}
