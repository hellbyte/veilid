use super::*;

mod test_types;

#[expect(clippy::unused_async)]
pub async fn test_all() {
    test_types::test_encrypted_value_data();
}
