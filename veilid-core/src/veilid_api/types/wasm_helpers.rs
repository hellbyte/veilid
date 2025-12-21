use super::*;

#[wasm_bindgen(typescript_custom_section)]
const OPAQUE_NEW_TYPES: &'static str = r#"
declare const OpaqueNewTypeSymbol: unique symbol
declare class OpaqueNewType<S extends symbol> {
    private [OpaqueNewTypeSymbol]: S
}
type Opaque<T, S extends symbol> = (T & OpaqueNewType<S>) | OpaqueNewType<S>
"#;

#[macro_export]
macro_rules! impl_opaque_newtype {
    ($name:ident, $base:ident) => {
        paste::paste! {
            #[wasm_bindgen(typescript_custom_section)]
            const [< IMPL_OPAQUE_NEW_TYPE_ $name:upper >]: &'static str = concat!(r#"
declare const "#, stringify!($name), r#"Symbol: unique symbol
export type "#, stringify!($name), r#" = Opaque<"#, stringify!($base), r#", typeof "#, stringify!($name), r#"Symbol>
"#);
        }
    };
}

#[macro_export]
macro_rules! make_wasm_bindgen_stubs {
    ($name:ident) => {
        paste::paste! {
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(typescript_type = $name)]
                pub type [< TypeStub $name >];
            }
        }
    };
}

pub mod ts {
    use super::*;

    /// Options that override defaults for set_dht_value
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    #[serde(rename_all = "camelCase")]
    pub struct SetDHTValueOptions {
        /// Override writer key pair for this operation
        #[tsify(type = "KeyPair", optional)]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub writer: JsValue,
        /// Defaults to true. If false, the value will not be written if the node is offline,
        /// and a TryAgain error will be returned.
        #[tsify(optional)]
        pub allow_offline: Option<AllowOffline>,
    }

    /// Options that override defaults for transact_dht_records
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    #[serde(rename_all = "camelCase")]
    pub struct TransactDHTRecordsOptions {
        #[tsify(type = "KeyPair", optional)]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub default_signing_keypair: JsValue,
    }

    /// Options that override defaults for DHTTransaction::set
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    pub struct DHTTransactionSetValueOptions {
        /// Override writer key pair for this operation
        #[tsify(type = "KeyPair", optional)]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub writer: JsValue,
    }

    /// Simple DHT Schema (SMPL) Member
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    #[serde(rename_all = "camelCase")]
    pub struct DHTSchemaSMPLMember {
        /// Member key
        #[tsify(type = "BareMemberId")]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub m_key: JsValue,
        /// Member subkey count
        pub m_cnt: u16,
    }

    /// Simple DHT Schema (SMPL)
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    #[serde(rename_all = "camelCase")]
    pub struct DHTSchemaSMPL {
        /// Owner subkey count
        pub o_cnt: u16,
        /// Members
        pub members: Vec<DHTSchemaSMPLMember>,
    }

    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    pub enum DHTSchema {
        DFLT(DHTSchemaDFLT),
        SMPL(DHTSchemaSMPL),
    }

    /// DHT Record Descriptor
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    #[serde(rename_all = "camelCase")]
    pub struct DHTRecordDescriptor {
        /// DHT Key = Hash(ownerKeyKind) of: [ ownerKeyValue, schema ]
        #[tsify(type = "RecordKey")]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub key: JsValue,
        /// The public key of the owner
        /// If this key is being created: KeyPair
        /// If this key is just being opened: PublicKey
        #[tsify(type = "PublicKey | KeyPair")]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub owner: JsValue,
        /// The schema in use associated with the key
        pub schema: DHTSchema,
    }

    /// A DHT value and its metadata
    #[derive(Serialize, Deserialize, Clone, Tsify)]
    #[tsify(from_wasm_abi, into_wasm_abi)]
    #[serde(rename_all = "camelCase")]
    pub struct ValueData {
        /// An increasing sequence number to time-order the DHT record changes
        pub seq: ValueSeqNum,

        /// The contents of a DHT Record
        #[cfg_attr(
            all(target_arch = "wasm32", target_os = "unknown"),
            serde(with = "serde_bytes"),
            tsify(type = "Uint8Array")
        )]
        pub data: Vec<u8>,

        /// The public identity key of the writer of the data
        #[tsify(type = "PublicKey")]
        #[serde(with = "serde_wasm_bindgen::preserve")]
        pub writer: JsValue,
    }
}

impl TryFrom<ts::SetDHTValueOptions> for SetDHTValueOptions {
    type Error = VeilidAPIError;

    fn try_from(value: ts::SetDHTValueOptions) -> Result<Self, Self::Error> {
        let writer = wasm_bindgen_derive::try_from_js_option::<KeyPair>(value.writer)
            .map_err(VeilidAPIError::generic)?;
        let allow_offline = value.allow_offline.clone();
        Ok(SetDHTValueOptions {
            writer,
            allow_offline,
        })
    }
}

impl TryFrom<ts::TransactDHTRecordsOptions> for TransactDHTRecordsOptions {
    type Error = VeilidAPIError;

    fn try_from(value: ts::TransactDHTRecordsOptions) -> Result<Self, Self::Error> {
        let default_signing_keypair =
            wasm_bindgen_derive::try_from_js_option::<KeyPair>(value.default_signing_keypair)
                .map_err(VeilidAPIError::generic)?;
        Ok(TransactDHTRecordsOptions {
            default_signing_keypair,
        })
    }
}

impl TryFrom<ts::DHTTransactionSetValueOptions> for DHTTransactionSetValueOptions {
    type Error = VeilidAPIError;

    fn try_from(value: ts::DHTTransactionSetValueOptions) -> Result<Self, Self::Error> {
        let writer = wasm_bindgen_derive::try_from_js_option::<KeyPair>(value.writer)
            .map_err(VeilidAPIError::generic)?;
        Ok(DHTTransactionSetValueOptions { writer })
    }
}

impl TryFrom<ts::DHTSchemaSMPLMember> for DHTSchemaSMPLMember {
    type Error = VeilidAPIError;

    fn try_from(value: ts::DHTSchemaSMPLMember) -> Result<Self, Self::Error> {
        let Some(m_key) =
            wasm_bindgen_derive::try_from_js_option::<BareMemberId>(value.m_key.clone())
                .map_err(VeilidAPIError::generic)?
        else {
            apibail_invalid_argument!(
                "m_key must not be undefined",
                "m_key",
                value.m_key.as_string().unwrap_or_default()
            );
        };
        let m_cnt = value.m_cnt;
        Ok(DHTSchemaSMPLMember { m_key, m_cnt })
    }
}

impl From<DHTSchemaSMPLMember> for ts::DHTSchemaSMPLMember {
    fn from(value: DHTSchemaSMPLMember) -> Self {
        ts::DHTSchemaSMPLMember {
            m_key: value.m_key.into(),
            m_cnt: value.m_cnt,
        }
    }
}

impl TryFrom<ts::DHTSchemaSMPL> for DHTSchemaSMPL {
    type Error = VeilidAPIError;

    fn try_from(value: ts::DHTSchemaSMPL) -> Result<Self, Self::Error> {
        let o_cnt = value.o_cnt;
        let mut members = vec![];
        for member in value.members {
            members.push(member.try_into().map_err(VeilidAPIError::generic)?);
        }
        DHTSchemaSMPL::new(o_cnt, members)
    }
}

impl From<DHTSchemaSMPL> for ts::DHTSchemaSMPL {
    fn from(value: DHTSchemaSMPL) -> Self {
        let mut members = vec![];
        for member in value.members() {
            members.push(member.clone().into())
        }

        ts::DHTSchemaSMPL {
            o_cnt: value.o_cnt(),
            members,
        }
    }
}

impl TryFrom<ts::DHTSchema> for DHTSchema {
    type Error = VeilidAPIError;

    fn try_from(value: ts::DHTSchema) -> Result<Self, Self::Error> {
        match value {
            ts::DHTSchema::DFLT(d) => Ok(DHTSchema::DFLT(d)),
            ts::DHTSchema::SMPL(s) => Ok(DHTSchema::SMPL(s.try_into()?)),
        }
    }
}

impl From<DHTSchema> for ts::DHTSchema {
    fn from(value: DHTSchema) -> Self {
        match value {
            DHTSchema::DFLT(d) => ts::DHTSchema::DFLT(d),
            DHTSchema::SMPL(s) => ts::DHTSchema::SMPL(s.into()),
        }
    }
}

impl TryFrom<ts::DHTRecordDescriptor> for DHTRecordDescriptor {
    type Error = VeilidAPIError;

    fn try_from(value: ts::DHTRecordDescriptor) -> Result<Self, Self::Error> {
        let Some(key) = wasm_bindgen_derive::try_from_js_option::<RecordKey>(value.key.clone())
            .map_err(VeilidAPIError::generic)?
        else {
            apibail_invalid_argument!(
                "key must not be undefined",
                "key",
                value.key.as_string().unwrap_or_default()
            );
        };
        let (owner, owner_secret) =
            match wasm_bindgen_derive::try_from_js_option::<KeyPair>(value.owner.clone()) {
                Ok(Some(v)) => (v.key(), Some(v.secret())),
                Ok(None) => {
                    apibail_invalid_argument!(
                        "owner must not be undefined",
                        "owner",
                        value.owner.as_string().unwrap_or_default()
                    );
                }
                Err(_) => {
                    match wasm_bindgen_derive::try_from_js_option::<PublicKey>(value.owner.clone())
                    {
                        Ok(Some(v)) => (v, None),
                        Err(e) => apibail_generic!(e),
                        Ok(None) => {
                            apibail_invalid_argument!(
                                "owner must not be undefined",
                                "owner",
                                value.owner.as_string().unwrap_or_default()
                            );
                        }
                    }
                }
            };

        let schema = value.schema.try_into()?;

        Ok(DHTRecordDescriptor::new(key, owner, owner_secret, schema))
    }
}

impl From<DHTRecordDescriptor> for ts::DHTRecordDescriptor {
    fn from(value: DHTRecordDescriptor) -> Self {
        ts::DHTRecordDescriptor {
            key: value.key().into(),
            owner: match value.owner_keypair() {
                Some(owner_keypair) => owner_keypair.into(),
                None => value.owner().into(),
            },
            schema: value.schema().into(),
        }
    }
}

impl From<ValueData> for ts::ValueData {
    fn from(value: ValueData) -> Self {
        ts::ValueData {
            seq: value.seq(),
            data: value.data().to_vec(),
            writer: value.writer().into(),
        }
    }
}
