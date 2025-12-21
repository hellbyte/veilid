use super::*;

/// DHT Record Descriptor
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[must_use]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
pub struct DHTRecordDescriptor {
    /// DHT Key = Hash(ownerKeyKind) of: [ ownerKeyValue, schema ]
    #[schemars(with = "String")]
    key: RecordKey,
    /// The public key of the owner
    #[schemars(with = "String")]
    owner: PublicKey,
    /// If this key is being created: Some(the secret key of the owner)
    /// If this key is just being opened: None
    #[schemars(with = "Option<String>")]
    owner_secret: Option<SecretKey>,
    /// The schema in use associated with the key
    schema: DHTSchema,
}

impl DHTRecordDescriptor {
    pub(crate) fn new(
        key: RecordKey,
        owner: PublicKey,
        owner_secret: Option<SecretKey>,
        schema: DHTSchema,
    ) -> Self {
        if let Some(owner_secret) = &owner_secret {
            assert_eq!(owner_secret.kind(), owner.kind());
        }
        Self {
            key,
            owner,
            owner_secret,
            schema,
        }
    }
    pub fn ref_key(&self) -> &RecordKey {
        &self.key
    }
    pub fn ref_owner(&self) -> &PublicKey {
        &self.owner
    }
    #[must_use]
    pub fn ref_owner_secret(&self) -> Option<&SecretKey> {
        self.owner_secret.as_ref()
    }
    pub fn ref_schema(&self) -> &DHTSchema {
        &self.schema
    }

    pub fn key(&self) -> RecordKey {
        self.key.clone()
    }

    pub fn owner(&self) -> PublicKey {
        self.owner.clone()
    }

    #[must_use]
    pub fn owner_secret(&self) -> Option<SecretKey> {
        self.owner_secret.clone()
    }

    pub fn schema(&self) -> DHTSchema {
        self.schema.clone()
    }

    #[must_use]
    pub fn owner_keypair(&self) -> Option<KeyPair> {
        self.owner_secret
            .as_ref()
            .map(|s| KeyPair::new_from_parts(self.owner.clone(), s.ref_value().clone()))
    }
}
