use super::*;
mod test_signed_value_data;

#[expect(clippy::unused_async)]
pub async fn test_all() {
    test_signed_value_data::test_encode_and_decode_signed_value_data();
}
