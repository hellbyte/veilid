#[macro_export]
macro_rules! impl_crypto_typed {
    ($visibility:vis $name:ident) => {
        paste::paste! {

            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(wasm_bindgen_derive::TryFromJsValue))]
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
            #[derive(Clone, PartialEq, Eq, Hash, GetSize)]
            #[must_use]
            $visibility struct $name
            {
                kind: CryptoKind,
                value: [<Bare $name>],
            }

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            make_wasm_bindgen_stubs!($name);

            impl $name {
                pub fn new(kind: CryptoKind, value: [<Bare $name>]) -> Self {
                    Self { kind, value }
                }

                pub fn ref_value(&self) -> &[<Bare $name>] {
                    &self.value
                }
                #[allow(dead_code)]
                pub fn into_value(self) -> [<Bare $name>] {
                    self.value
                }
            }

            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
            impl $name {
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter, unchecked_return_type = "CryptoKind"))]
                pub fn kind(&self) -> CryptoKind {
                    self.kind
                }
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter))]
                #[allow(dead_code)]
                pub fn value(&self) -> [<Bare $name>] {
                    self.value.clone()
                }
            }

            impl PartialOrd for $name
            {
                fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl Ord for $name
            {
                fn cmp(&self, other: &Self) -> cmp::Ordering {
                    let x = compare_crypto_kind(&self.kind, &other.kind);
                    if x != cmp::Ordering::Equal {
                        return x;
                    }
                    self.value.cmp(&other.value)
                }
            }

            impl fmt::Display for $name
            {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                    write!(f, "{}:{}", self.kind, self.value)
                }
            }

            impl fmt::Debug for $name
            {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                    write!(f, concat!(stringify!($name), "("))?;
                    write!(f, "{}", self.value)?;
                    write!(f, ")")
                }
            }

            impl FromStr for $name
            {
                type Err = VeilidAPIError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let b = s.as_bytes();
                    if b.len() > 5 && b[4..5] == b":"[..] {
                        let kind: CryptoKind = b[0..4].try_into().expect_or_log("should not fail to convert");
                        let value = [<Bare $name>]::try_decode_bytes(&b[5..])?;
                        Ok(Self { kind, value })
                    } else {
                        let kind = best_crypto_kind();
                        let value = [<Bare $name>]::try_decode_bytes(b)?;
                        Ok(Self { kind, value })
                    }
                }
            }

            impl TryFrom<String> for $name
            {
                type Error = VeilidAPIError;

                fn try_from(s: String) -> Result<Self, Self::Error> {
                    Self::from_str(&s)
                }
            }

            impl TryFrom<&str> for $name
            {
                type Error = VeilidAPIError;

                fn try_from(s: &str) -> Result<Self, Self::Error> {
                    Self::from_str(s)
                }
            }

            impl<'de> Deserialize<'de> for $name
            {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let s = <String as Deserialize>::deserialize(deserializer)?;
                    FromStr::from_str(&s).map_err(serde::de::Error::custom)
                }
            }
            impl Serialize for $name
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.collect_str(self)
                }
            }

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            #[wasm_bindgen]
            impl $name {
                #[wasm_bindgen(constructor)]
                pub fn js_new(kind: CryptoKind, value: [<Bare $name>]) -> Self {
                    Self::new(kind,value)
                }

                #[wasm_bindgen(js_name = parse)]
                pub fn js_parse(s: String) -> VeilidAPIResult<Self> {
                    Self::from_str(&s)
                }

                #[wasm_bindgen(js_name = toString)]
                pub fn js_to_string(&self) -> String {
                    self.to_string()
                }

                #[wasm_bindgen(js_name = isEqual)]
                pub fn js_is_equal(&self, other: &Self) -> bool {
                    self == other
                }
                // TODO: add more typescript-only operations here
            }
        }
    };
}

#[macro_export]
macro_rules! impl_crypto_typed_vec {
    ($visibility:vis $name:ident) => {
        paste::paste! {
            impl<'a> TryFrom<&'a [u8]> for $name
            {
                type Error = VeilidAPIError;

                fn try_from(b: &'a [u8]) -> Result<Self, Self::Error> {
                    if b.len() < 4 {
                        apibail_generic!("invalid cryptotyped format");
                    }
                    let kind: CryptoKind = b[0..4].try_into()?;
                    let value: [<Bare $name>] = b[4..].into();
                    Ok(Self { kind, value })
                }
            }

            impl TryFrom<Vec<u8>> for $name
            {
                type Error = VeilidAPIError;

                fn try_from(b: Vec<u8>) -> Result<Self, Self::Error> {
                    Self::try_from(b.as_slice())
                }
            }

            impl From<$name> for Vec<u8>
            {
                fn from(v: $name) -> Self {
                    let mut out = v.kind.0.to_vec();
                    out.extend_from_slice(v.value.as_ref());
                    out
                }
            }

            impl From<&$name> for Vec<u8>
            {
                fn from(v: &$name) -> Self {
                    let mut out = v.kind.0.to_vec();
                    out.extend_from_slice(v.value.as_ref());
                    out
                }
            }
        }
    };
}
