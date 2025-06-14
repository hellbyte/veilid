use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[must_use]
pub struct CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
{
    pub kind: CryptoKind,
    pub value: K,
}

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
        #[wasm_bindgen(typescript_custom_section)]
        const CRYPOTYPED_TYPE: &'static str = r#"
export type CryptoTyped<TCryptoKey extends string> = `${CryptoKind}:${TCryptoKey}`;
"#;
    }
}

impl<K> CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
{
    pub fn new(kind: CryptoKind, value: K) -> Self {
        Self { kind, value }
    }
}
impl<K> PartialOrd for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: Ord + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: Ord + PartialOrd,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let x = compare_crypto_kind(&self.kind, &other.kind);
        if x != cmp::Ordering::Equal {
            return x;
        }
        self.value.cmp(&other.value)
    }
}

impl<K> fmt::Display for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.kind, self.value)
    }
}

impl<K> FromStr for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: Encodable,
{
    type Err = VeilidAPIError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.as_bytes();
        if b.len() == (5 + K::encoded_len()) && b[4..5] == b":"[..] {
            let kind: CryptoKind = b[0..4].try_into().expect("should not fail to convert");
            let value = K::try_decode_bytes(&b[5..])?;
            Ok(Self { kind, value })
        } else if b.len() == K::encoded_len() {
            let kind = best_crypto_kind();
            let value = K::try_decode_bytes(b)?;
            Ok(Self { kind, value })
        } else {
            apibail_generic!("invalid cryptotyped format");
        }
    }
}

impl<K> TryFrom<String> for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: Encodable,
{
    type Error = VeilidAPIError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl<K> TryFrom<&str> for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: Encodable,
{
    type Error = VeilidAPIError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::from_str(s)
    }
}

impl<'a, K> TryFrom<&'a [u8]> for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: TryFrom<&'a [u8], Error = VeilidAPIError>,
{
    type Error = VeilidAPIError;

    fn try_from(b: &'a [u8]) -> Result<Self, Self::Error> {
        if b.len() < 4 {
            apibail_generic!("invalid cryptotyped format");
        }
        let kind: CryptoKind = b[0..4].try_into()?;
        let value: K = b[4..].try_into()?;
        Ok(Self { kind, value })
    }
}

impl<K> TryFrom<Vec<u8>> for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: for<'a> TryFrom<&'a [u8], Error = VeilidAPIError>,
{
    type Error = VeilidAPIError;

    fn try_from(b: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(b.as_slice())
    }
}

impl<K> From<CryptoTyped<K>> for Vec<u8>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: AsRef<[u8]>,
{
    fn from(v: CryptoTyped<K>) -> Self {
        let mut out = v.kind.0.to_vec();
        out.extend_from_slice(v.value.as_ref());
        out
    }
}

impl<'de, K> Deserialize<'de> for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: Encodable,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}
impl<K> Serialize for CryptoTyped<K>
where
    K: Clone + Copy + fmt::Debug + PartialEq + Eq + Hash,
    K: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}
