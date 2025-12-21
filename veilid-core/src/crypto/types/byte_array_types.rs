use super::*;

use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use core::convert::TryFrom;
use core::fmt;
use core::hash::Hash;

use bytes::{Bytes, BytesMut};
use data_encoding::BASE64URL_NOPAD;

//////////////////////////////////////////////////////////////////////

fn bytes_size_helper(bytes: &Bytes) -> usize {
    bytes.len()
}

macro_rules! byte_array_type {
    ($visibility:vis $name:ident) => {

        #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(wasm_bindgen_derive::TryFromJsValue))]
        #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
        #[derive(Clone, Hash, Default, PartialOrd, Ord, PartialEq, Eq, GetSize)]
        #[must_use]
        $visibility struct $name {
            #[get_size(size_fn = bytes_size_helper)]
            bytes: Bytes,
        }


        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        make_wasm_bindgen_stubs!($name);

        impl $name {
            pub fn new(data: &[u8]) -> Self {
                Self {
                    bytes: Bytes::copy_from_slice(data),
                }
            }
            fn new_from_bytes(bytes: Bytes) -> Self {
                Self { bytes }
            }

            pub fn bytes(&self) -> &[u8] {
                &self.bytes
            }

            #[allow(dead_code)]
            pub fn first_nonzero_nibble(&self) -> Option<(usize, u8)> {
                for i in 0..(self.bytes.len() * 2) {
                    let n = self.nibble(i);
                    if n != 0 {
                        return Some((i, n));
                    }
                }
                None
            }
        }
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn js_new(data: &[u8]) -> Self {
                Self::new(data)
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

        #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
        #[allow(dead_code)]
        impl $name {
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter, js_name = length))]
            pub fn len(&self) -> usize {
                self.bytes.len()
            }
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = isEmpty))]
            pub fn is_empty(&self) -> bool {
                self.bytes.is_empty()
            }
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = toArray))]
            pub fn to_vec(&self) -> Vec<u8> {
                self.bytes.to_vec()
            }
            // Big endian bit ordering
            pub fn bit(&self, index: usize) -> bool {
                let bi = index / 8;
                let ti = 7 - (index % 8);
                ((self.bytes[bi] >> ti) & 1) != 0
            }

            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter, js_name = "firstNonZeroBit"))]
            pub fn first_nonzero_bit(&self) -> Option<usize> {
                for i in 0..self.bytes.len() {
                    let b = self.bytes[i];
                    if b != 0 {
                        for n in 0..8 {
                            if ((b >> (7 - n)) & 1u8) != 0u8 {
                                return Some((i * 8) + n);
                            }
                        }
                        unreachable!("nonzero byte must have nonzero bit");
                    }
                }
                None
            }

            // Big endian nibble ordering
            pub fn nibble(&self, index: usize) -> u8 {
                let bi = index / 2;
                if index & 1 == 0 {
                    (self.bytes[bi] >> 4) & 0xFu8
                } else {
                    self.bytes[bi] & 0xFu8
                }
            }

            pub fn encode(&self) -> String {
                BASE64URL_NOPAD.encode(&self.bytes)
            }
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter, js_name = "encodedLength"))]
            pub fn encoded_len(&self) -> usize {
                BASE64URL_NOPAD.encode_len(self.bytes.len())
            }
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = tryDecode))]
            pub fn try_decode(input: &str) -> VeilidAPIResult<Self> {
                let b = input.as_bytes();
                Self::try_decode_bytes(b)
            }
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = tryDecodeBytes))]
            pub fn try_decode_bytes(b: &[u8]) -> VeilidAPIResult<Self> {
                if b.len() == 0 {
                    return Ok(Self::default());
                }
                let decode_len = BASE64URL_NOPAD
                    .decode_len(b.len())
                    .map_err(|_| VeilidAPIError::generic("failed to get decode length"))?;
                let mut bytes = BytesMut::zeroed(decode_len);
                let bytes_len = BASE64URL_NOPAD
                    .decode_mut(b, &mut bytes)
                    .map_err(|_| VeilidAPIError::generic("failed to decode"))?;
                bytes.truncate(bytes_len);
                Ok(Self::new_from_bytes(bytes.freeze()))
            }
        }

        impl core::ops::Deref for $name {
            type Target = [u8];

            fn deref(&self) -> &Self::Target {
                &self.bytes
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let s = self.encode();
                serde::Serialize::serialize(&s, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = <String as serde::Deserialize>::deserialize(deserializer)?;
                Self::try_decode(s.as_str()).map_err(serde::de::Error::custom)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.encode())
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "("))?;
                write!(f, "{}", self.encode())?;
                write!(f, ")")
            }
        }

        impl From<&$name> for String {
            fn from(value: &$name) -> Self {
                value.encode()
            }
        }

        impl FromStr for $name {
            type Err = VeilidAPIError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $name::try_from(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = VeilidAPIError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl TryFrom<&str> for $name {
            type Error = VeilidAPIError;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_decode(value)
            }
        }

        impl TryFrom<$name> for Vec<u8> {
            type Error = VeilidAPIError;
            fn try_from(value: $name) -> Result<Self, Self::Error> {
                Ok(value.bytes().to_vec())
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.bytes()
            }
        }
        impl From<&[u8]> for $name {
            fn from(v: &[u8]) -> Self {
                Self {
                    bytes: Bytes::copy_from_slice(v),
                }
            }
        }
        impl From<Vec<u8>> for $name {
            fn from(v: Vec<u8>) -> Self {
                Self {
                    bytes: Bytes::from(v),
                }
            }
        }
    }
}

/////////////////////////////////////////

// Untyped public key (variable length)
byte_array_type!(pub BarePublicKey);
// Untyped secret key (variable length)
byte_array_type!(pub BareSecretKey);
// Untyped encapsulation key (variable length)
byte_array_type!(pub BareEncapsulationKey);
// Untyped decapsulation key (variable length)
byte_array_type!(pub BareDecapsulationKey);
// Untyped signature (variable length)
byte_array_type!(pub BareSignature);
// Untyped hash digest (hashed to 32 bytes)
byte_array_type!(pub BareHashDigest);
// Untyped shared secret (variable length)
byte_array_type!(pub BareSharedSecret);
// Untyped record key (hashed to 32 bytes)
byte_array_type!(pub BareOpaqueRecordKey);
// Untyped route id (hashed to 32 bytes)
byte_array_type!(pub BareRouteId);
// Untyped node id (hashed to 32 bytes)
byte_array_type!(pub BareNodeId);
// Untyped member id (hashed to 32 bytes)
byte_array_type!(pub BareMemberId);
// Untyped nonce (random 24 bytes, no typed variant)
byte_array_type!(pub Nonce);

// Internal types
byte_array_type!(pub(crate) BareHashCoordinate);
byte_array_type!(pub(crate) HashDistance);
