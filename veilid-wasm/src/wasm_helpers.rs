#[allow(unused_imports)]
use super::*;

// JSON Helpers for WASM
#[allow(dead_code)]
pub fn to_json<T: Serialize + Debug>(val: T) -> JsValue {
    JsValue::from_str(&serialize_json(val))
}

pub fn to_jsvalue<T>(val: T) -> JsValue
where
    JsValue: From<T>,
{
    JsValue::from(val)
}

#[expect(dead_code)]
pub fn from_json<T: de::DeserializeOwned + Debug>(
    val: JsValue,
) -> Result<T, veilid_core::VeilidAPIError> {
    let s = val
        .as_string()
        .ok_or_else(|| veilid_core::VeilidAPIError::ParseError {
            message: "Value is not String".to_owned(),
            value: String::new(),
        })?;
    deserialize_json(&s)
}

// Marshalling helpers
#[expect(dead_code)]
pub fn unmarshall(b64: String) -> VeilidAPIResult<Vec<u8>> {
    data_encoding::BASE64URL_NOPAD
        .decode(b64.as_bytes())
        .map_err(|e| {
            VeilidAPIError::generic(format!(
                "error decoding base64url string '{}' into bytes: {}",
                b64, e
            ))
        })
}

#[expect(dead_code)]
#[must_use]
pub fn marshall(data: &[u8]) -> String {
    data_encoding::BASE64URL_NOPAD.encode(data)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Uint8Array[]")]
    pub type Uint8ArrayArray;
}

/// Convert a `Vec<Uint8Array>` into a `js_sys::Array` with the type of `Uint8Array[]`
#[allow(dead_code)]
pub(crate) fn into_unchecked_uint8array_array(items: Vec<Uint8Array>) -> Uint8ArrayArray {
    items
        .iter()
        .collect::<js_sys::Array>()
        .unchecked_into::<Uint8ArrayArray>() // TODO: can I do this a better way?
}
