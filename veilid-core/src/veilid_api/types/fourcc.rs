#[macro_export]
macro_rules! fourcc_type {
    ($name:ident) => {
        paste::paste! {
            /// A four-character code
            #[derive(
                Copy,
                Default,
                Clone,
                Hash,
                PartialOrd,
                Ord,
                PartialEq,
                Eq,
                Serialize,
                Deserialize,
                JsonSchema,
                GetSize,
            )]
            #[serde(try_from = "String", into = "String")]
            #[must_use]
            #[cfg_attr(
                all(target_arch = "wasm32", target_os = "unknown"),
                derive(Tsify),
                tsify(into_wasm_abi, from_wasm_abi, type_suffix = "Inner"),
            )]
            #[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
            pub struct $name(#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"),tsify(type = "string"))] [u8; 4]);

            impl $name {
                pub const fn new(b: [u8; 4]) -> Self {
                    $name(b)
                }
                #[must_use]
                pub fn bytes(&self) -> &[u8; 4] {
                    &self.0
                }
            }

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            impl_opaque_newtype!($name, [< $name Inner >]);

            impl From<[u8; 4]> for $name {
                fn from(b: [u8; 4]) -> Self {
                    Self(b)
                }
            }

            impl From<u32> for $name {
                fn from(u: u32) -> Self {
                    Self(u.to_be_bytes())
                }
            }

            impl From<$name> for u32 {
                fn from(u: $name) -> Self {
                    u32::from_be_bytes(u.0)
                }
            }

            impl From<$name> for String {
                fn from(u: $name) -> Self {
                    String::from_utf8_lossy(&u.0).to_string()
                }
            }

            impl TryFrom<&[u8]> for $name {
                type Error = VeilidAPIError;
                fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
                    Ok(Self(b.try_into().map_err(VeilidAPIError::generic)?))
                }
            }

            impl TryFrom<String> for $name {
                type Error = VeilidAPIError;
                fn try_from(s: String) -> Result<Self, Self::Error> {
                    Self::from_str(s.as_str())
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                    write!(f, "{}", String::from_utf8_lossy(&self.0))
                }
            }
            impl fmt::Debug for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                    write!(f, "{}", String::from_utf8_lossy(&self.0))
                }
            }

            impl FromStr for $name {
                type Err = VeilidAPIError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Ok(Self(
                        s.as_bytes().try_into().map_err(VeilidAPIError::generic)?,
                    ))
                }
            }
        }
    };
}
