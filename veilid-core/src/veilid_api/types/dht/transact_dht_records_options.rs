use super::*;

/// Options for DHT record transactions
#[derive(Debug, JsonSchema, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
pub struct TransactDHTRecordsOptions {
    /// The signing keypair to use when opening the transaction.
    /// Setting this does not override any writer keys used by transaction operations.
    /// If a record in the transaction is already opened for writing then the writer key will be used.
    /// This is only useful if you have records in a transaction that are only open for reading.
    #[schemars(with = "Option<String>")]
    pub default_signing_keypair: Option<KeyPair>,
}
