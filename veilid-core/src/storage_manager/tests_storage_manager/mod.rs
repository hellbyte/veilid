use super::*;

mod test_encrypted_value_data;

pub async fn test_all() {
    test_encrypted_value_data::test_all().await;
    record_store::tests::test_record_store().await;
}
