use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CryptoSystemRequest {
    pub cs_id: u32,
    #[serde(flatten)]
    pub cs_op: CryptoSystemRequestOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CryptoSystemResponse {
    pub cs_id: u32,
    #[serde(flatten)]
    pub cs_op: CryptoSystemResponseOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "cs_op")]
pub enum CryptoSystemRequestOp {
    Release,
    Kind,
    CachedDh {
        #[schemars(with = "String")]
        key: PublicKey,
        #[schemars(with = "String")]
        secret: SecretKey,
    },
    ComputeDh {
        #[schemars(with = "String")]
        key: PublicKey,
        #[schemars(with = "String")]
        secret: SecretKey,
    },
    GenerateSharedSecret {
        #[schemars(with = "String")]
        key: PublicKey,
        #[schemars(with = "String")]
        secret: SecretKey,
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        domain: Vec<u8>,
    },
    RandomBytes {
        len: u32,
    },
    SharedSecretLength,
    NonceLength,
    HashDigestLength,
    PublicKeyLength,
    SecretKeyLength,
    SignatureLength,
    DefaultSaltLength,
    AeadOverhead,
    CheckSharedSecret {
        #[schemars(with = "String")]
        secret: SharedSecret,
    },
    CheckNonce {
        #[schemars(with = "String")]
        nonce: Nonce,
    },
    CheckHashDigest {
        #[schemars(with = "String")]
        digest: HashDigest,
    },
    CheckPublicKey {
        #[schemars(with = "String")]
        key: PublicKey,
    },
    CheckSecretKey {
        #[schemars(with = "String")]
        key: SecretKey,
    },
    CheckSignature {
        #[schemars(with = "String")]
        signature: Signature,
    },
    HashPassword {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        password: Vec<u8>,
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        salt: Vec<u8>,
    },
    VerifyPassword {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        password: Vec<u8>,
        password_hash: String,
    },
    DeriveSharedSecret {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        password: Vec<u8>,
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        salt: Vec<u8>,
    },
    RandomNonce,
    RandomSharedSecret,
    GenerateKeyPair,
    GenerateHash {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        data: Vec<u8>,
    },
    ValidateKeyPair {
        #[schemars(with = "String")]
        key: PublicKey,
        #[schemars(with = "String")]
        secret: SecretKey,
    },
    ValidateHash {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        data: Vec<u8>,
        #[schemars(with = "String")]
        hash_digest: HashDigest,
    },
    Sign {
        #[schemars(with = "String")]
        key: PublicKey,
        #[schemars(with = "String")]
        secret: SecretKey,
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        data: Vec<u8>,
    },
    Verify {
        #[schemars(with = "String")]
        key: PublicKey,
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        data: Vec<u8>,
        #[schemars(with = "String")]
        signature: Signature,
    },
    DecryptAead {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        body: Vec<u8>,
        #[schemars(with = "String")]
        nonce: Nonce,
        #[schemars(with = "String")]
        shared_secret: SharedSecret,
        #[serde(with = "as_human_opt_base64")]
        #[schemars(with = "Option<String>")]
        associated_data: Option<Vec<u8>>,
    },
    EncryptAead {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        body: Vec<u8>,
        #[schemars(with = "String")]
        nonce: Nonce,
        #[schemars(with = "String")]
        shared_secret: SharedSecret,
        #[serde(with = "as_human_opt_base64")]
        #[schemars(with = "Option<String>")]
        associated_data: Option<Vec<u8>>,
    },
    CryptNoAuth {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        body: Vec<u8>,
        #[schemars(with = "String")]
        nonce: Nonce,
        #[schemars(with = "String")]
        shared_secret: SharedSecret,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "cs_op")]
pub enum CryptoSystemResponseOp {
    InvalidId,
    Release,
    Kind {
        #[schemars(with = "String")]
        value: CryptoKind,
    },
    CachedDh {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithString<SharedSecret>,
    },
    ComputeDh {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithString<SharedSecret>,
    },
    GenerateSharedSecret {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithString<SharedSecret>,
    },
    RandomBytes {
        #[serde(with = "as_human_base64")]
        #[schemars(with = "String")]
        value: Vec<u8>,
    },
    SharedSecretLength {
        value: u32,
    },
    NonceLength {
        value: u32,
    },
    HashDigestLength {
        value: u32,
    },
    PublicKeyLength {
        value: u32,
    },
    SecretKeyLength {
        value: u32,
    },
    SignatureLength {
        value: u32,
    },
    DefaultSaltLength {
        value: u32,
    },
    AeadOverhead {
        value: u32,
    },
    CheckSharedSecret {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    CheckNonce {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    CheckHashDigest {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    CheckPublicKey {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    CheckSecretKey {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    CheckSignature {
        #[serde(flatten)]
        result: ApiResult<()>,
    },
    HashPassword {
        #[serde(flatten)]
        result: ApiResult<String>,
    },
    VerifyPassword {
        #[serde(flatten)]
        result: ApiResult<bool>,
    },
    DeriveSharedSecret {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithString<SharedSecret>,
    },
    RandomNonce {
        #[schemars(with = "String")]
        value: Nonce,
    },
    RandomSharedSecret {
        #[schemars(with = "String")]
        value: SharedSecret,
    },
    GenerateKeyPair {
        #[schemars(with = "String")]
        value: KeyPair,
    },
    GenerateHash {
        #[schemars(with = "String")]
        value: HashDigest,
    },
    ValidateKeyPair {
        #[serde(flatten)]
        result: ApiResult<bool>,
    },
    ValidateHash {
        #[serde(flatten)]
        result: ApiResult<bool>,
    },
    Sign {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithString<Signature>,
    },
    Verify {
        #[serde(flatten)]
        result: ApiResult<bool>,
    },
    DecryptAead {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithVecU8,
    },
    EncryptAead {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithVecU8,
    },
    CryptNoAuth {
        #[serde(flatten)]
        #[schemars(with = "ApiResult<String>")]
        result: ApiResultWithVecU8,
    },
}
