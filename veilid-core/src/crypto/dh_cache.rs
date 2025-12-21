use super::*;
use crate::*;

// Diffie-Hellman key agreement cache
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DHCacheKey {
    pub key: PublicKey,
    pub secret: SecretKey,
}

impl fmt::Display for DHCacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.key, self.secret)
    }
}

impl FromStr for DHCacheKey {
    type Err = VeilidAPIError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((pks, sks)) = s.split_once('.') else {
            apibail_parse_error!("s", s);
        };
        let key = PublicKey::from_str(pks)?;
        let secret = SecretKey::from_str(sks)?;
        Ok(DHCacheKey { key, secret })
    }
}

impl<'de> Deserialize<'de> for DHCacheKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for DHCacheKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DHCacheValue {
    pub shared_secret: SharedSecret,
}

pub type DHCache = LruCache<DHCacheKey, DHCacheValue>;
pub const DH_CACHE_SIZE: usize = 4096;

pub fn cache_to_bytes(cache: &DHCache) -> Vec<u8> {
    serialize_json_bytes(cache)
}

pub fn bytes_to_cache(bytes: &[u8]) -> EyreResult<DHCache> {
    deserialize_json_bytes(bytes).wrap_err("cache format invalid")
}
