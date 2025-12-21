use super::*;

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(wasm_bindgen_derive::TryFromJsValue)
)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash, GetSize)]
#[must_use]
pub struct BareRecordKey {
    key: BareOpaqueRecordKey,
    encryption_key: Option<BareSharedSecret>,
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
make_wasm_bindgen_stubs!(BareRecordKey);

impl BareRecordKey {
    pub fn new(key: BareOpaqueRecordKey, encryption_key: Option<BareSharedSecret>) -> Self {
        Self {
            key,
            encryption_key,
        }
    }
    pub fn ref_key(&self) -> &BareOpaqueRecordKey {
        &self.key
    }
    pub fn ref_encryption_key(&self) -> Option<&BareSharedSecret> {
        self.encryption_key.as_ref()
    }
    pub fn split(&self) -> (BareOpaqueRecordKey, Option<BareSharedSecret>) {
        (self.key.clone(), self.encryption_key.clone())
    }
    pub fn into_split(self) -> (BareOpaqueRecordKey, Option<BareSharedSecret>) {
        (self.key, self.encryption_key)
    }
}

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[allow(dead_code)]
impl BareRecordKey {
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter)
    )]
    pub fn key(&self) -> BareOpaqueRecordKey {
        self.key.clone()
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter, js_name = "encryptionKey")
    )]
    pub fn encryption_key(&self) -> Option<BareSharedSecret> {
        self.encryption_key.clone()
    }
    pub fn encode(&self) -> String {
        if let Some(encryption_key) = &self.encryption_key {
            format!("{}:{}", self.key.encode(), encryption_key.encode())
        } else {
            self.key.encode()
        }
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter, js_name = "encodedLen")
    )]
    pub fn encoded_len(&self) -> usize {
        if let Some(encryption_key) = &self.encryption_key {
            self.key.encoded_len() + 1 + encryption_key.encoded_len()
        } else {
            self.key.encoded_len()
        }
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(js_name = "tryDecode")
    )]
    pub fn try_decode(input: &str) -> VeilidAPIResult<Self> {
        let b = input.as_bytes();
        Self::try_decode_bytes(b)
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(js_name = "tryDecodeBytes")
    )]
    pub fn try_decode_bytes(b: &[u8]) -> VeilidAPIResult<Self> {
        let parts: Vec<_> = b.split(|x| *x == b':').collect();
        match parts[..] {
            [key] => {
                let key = BareOpaqueRecordKey::try_decode_bytes(key)?;
                Ok(BareRecordKey {
                    key,
                    encryption_key: None,
                })
            }
            [key, encryption_key] => {
                let key = BareOpaqueRecordKey::try_decode_bytes(key)?;
                let encryption_key = BareSharedSecret::try_decode_bytes(encryption_key)?;
                Ok(BareRecordKey {
                    key,
                    encryption_key: Some(encryption_key),
                })
            }
            _ => {
                apibail_parse_error!(
                    "input has incorrect parts",
                    format!("parts={}", parts.len())
                );
            }
        }
    }
}

impl fmt::Display for BareRecordKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl fmt::Debug for BareRecordKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BareRecordKey({})", self.encode())
    }
}

impl From<&BareRecordKey> for String {
    fn from(value: &BareRecordKey) -> Self {
        value.encode()
    }
}

impl FromStr for BareRecordKey {
    type Err = VeilidAPIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BareRecordKey::try_from(s)
    }
}

impl TryFrom<String> for BareRecordKey {
    type Error = VeilidAPIError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        BareRecordKey::try_from(value.as_str())
    }
}

impl TryFrom<&str> for BareRecordKey {
    type Error = VeilidAPIError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_decode(value)
    }
}

impl serde::Serialize for BareRecordKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.encode();
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for BareRecordKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(BareRecordKey::default());
        }
        BareRecordKey::try_decode(s.as_str()).map_err(serde::de::Error::custom)
    }
}

////////////////////////////////////////////////////////////////////////////

impl RecordKey {
    pub fn from_opaque(
        opaque_record_key: OpaqueRecordKey,
        encryption_key: Option<BareSharedSecret>,
    ) -> Self {
        RecordKey::new(
            opaque_record_key.kind(),
            BareRecordKey::new(opaque_record_key.into_value(), encryption_key),
        )
    }
    pub fn opaque(&self) -> OpaqueRecordKey {
        OpaqueRecordKey::new(self.kind, self.ref_value().key())
    }
    pub fn into_split(self) -> (OpaqueRecordKey, Option<SharedSecret>) {
        let kind = self.kind;
        let (bork, bss) = self.into_value().into_split();
        (
            OpaqueRecordKey::new(kind, bork),
            bss.map(|x| SharedSecret::new(kind, x)),
        )
    }
}

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[allow(dead_code)]
impl RecordKey {
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter, js_name = "encryptionKey")
    )]
    pub fn encryption_key(&self) -> Option<SharedSecret> {
        self.ref_value()
            .encryption_key()
            .map(|v| SharedSecret::new(self.kind, v.clone()))
    }
}
