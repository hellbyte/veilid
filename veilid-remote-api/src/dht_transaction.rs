use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DhtTransactionRequest {
    pub dhttx_id: u32,
    #[serde(flatten)]
    pub dhttx_op: DhtTransactionRequestOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DhtTransactionResponse {
    pub dhttx_id: u32,
    #[serde(flatten)]
    pub dhttx_op: DhtTransactionResponseOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "dhttx_op")]
pub enum DhtTransactionRequestOp {
    Release,
    Commit,
    Rollback,
    Get {
        #[schemars(with = "String")]
        key: RecordKey,
        subkey: ValueSubkey,
    },
    Set {
        #[schemars(with = "String")]
        key: RecordKey,
        subkey: ValueSubkey,
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        data: Vec<u8>,
        options: Option<DHTTransactionSetValueOptions>,
    },
    Inspect {
        #[schemars(with = "String")]
        key: RecordKey,
        subkeys: Option<ValueSubkeyRangeSet>,
        #[schemars(default)]
        scope: DHTReportScope,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "dhttx_op")]
pub enum DhtTransactionResponseOp {
    InvalidId,
    Release,
    Commit {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    Rollback {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    Get {
        #[serde(flatten)]
        result: ApiResult<Option<ValueData>>,
    },
    Set {
        #[serde(flatten)]
        result: ApiResult<Option<ValueData>>,
    },
    Inspect {
        #[serde(flatten)]
        result: ApiResult<Box<DHTRecordReport>>,
    },
}
