use crate::crypto::tests::fixtures::*;

use super::*;

// encrypted_value_data

pub fn test_encrypted_value_data() {
    let orig = EncryptedValueData::new(
        42.into(),
        b"Brent Spiner".to_vec(),
        fix_fake_public_key(),
        None,
    );
    let copy = deserialize_json(&serialize_json(&orig)).unwrap();

    assert_eq!(orig, copy);
}
