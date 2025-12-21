use super::*;

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(wasm_bindgen_derive::TryFromJsValue)
)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[derive(Clone, Default, Hash, PartialOrd, Ord, PartialEq, Eq, GetSize)]
#[must_use]
pub struct BareKeyPair {
    key: BarePublicKey,
    secret: BareSecretKey,
}

impl BareKeyPair {
    pub fn new(key: BarePublicKey, secret: BareSecretKey) -> Self {
        Self { key, secret }
    }
    pub fn ref_key(&self) -> &BarePublicKey {
        &self.key
    }
    pub fn ref_secret(&self) -> &BareSecretKey {
        &self.secret
    }
    pub fn ref_split(&self) -> (&BarePublicKey, &BareSecretKey) {
        (&self.key, &self.secret)
    }
    pub fn split(&self) -> (BarePublicKey, BareSecretKey) {
        (self.key.clone(), self.secret.clone())
    }
    pub fn into_split(self) -> (BarePublicKey, BareSecretKey) {
        (self.key, self.secret)
    }
}

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[allow(dead_code)]
impl BareKeyPair {
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter)
    )]
    pub fn key(&self) -> BarePublicKey {
        self.key.clone()
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter)
    )]
    pub fn secret(&self) -> BareSecretKey {
        self.secret.clone()
    }
    pub fn encode(&self) -> String {
        format!("{}:{}", self.key.encode(), self.secret.encode())
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter, js_name = "encodedLength")
    )]
    pub fn encoded_len(&self) -> usize {
        self.key.encoded_len() + 1 + self.secret.encoded_len()
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
        if parts.len() != 2 {
            apibail_parse_error!(
                "input has incorrect parts",
                format!("parts={}", parts.len())
            );
        }
        let key = BarePublicKey::try_decode_bytes(parts[0])?;
        let secret = BareSecretKey::try_decode_bytes(parts[1])?;
        Ok(BareKeyPair { key, secret })
    }
}

impl fmt::Display for BareKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl fmt::Debug for BareKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BareKeyPair({})", self.encode())
    }
}

impl From<&BareKeyPair> for String {
    fn from(value: &BareKeyPair) -> Self {
        value.encode()
    }
}

impl FromStr for BareKeyPair {
    type Err = VeilidAPIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BareKeyPair::try_from(s)
    }
}

impl TryFrom<String> for BareKeyPair {
    type Error = VeilidAPIError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        BareKeyPair::try_from(value.as_str())
    }
}

impl TryFrom<&str> for BareKeyPair {
    type Error = VeilidAPIError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_decode(value)
    }
}

impl serde::Serialize for BareKeyPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.encode();
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for BareKeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(BareKeyPair::default());
        }
        BareKeyPair::try_decode(s.as_str()).map_err(serde::de::Error::custom)
    }
}

////////////////////////////////////////////////////////////////////////////

impl KeyPair {
    pub fn into_split(self) -> (PublicKey, SecretKey) {
        let kind = self.kind;
        let (pk, sk) = self.into_value().into_split();
        (PublicKey::new(kind, pk), SecretKey::new(kind, sk))
    }

    pub fn ref_bare_secret(&self) -> &BareSecretKey {
        self.ref_value().ref_secret()
    }
}

#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
#[allow(dead_code)]
impl KeyPair {
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(js_name = "newFromParts")
    )]
    pub fn new_from_parts(key: PublicKey, bare_secret: BareSecretKey) -> Self {
        Self {
            kind: key.kind(),
            value: BareKeyPair::new(key.value(), bare_secret),
        }
    }

    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter)
    )]
    pub fn key(&self) -> PublicKey {
        PublicKey::new(self.kind, self.ref_value().key())
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter)
    )]
    pub fn secret(&self) -> SecretKey {
        SecretKey::new(self.kind, self.ref_value().secret())
    }
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        wasm_bindgen(getter, js_name = "bareSecret")
    )]
    pub fn bare_secret(&self) -> BareSecretKey {
        self.ref_value().secret()
    }
}
