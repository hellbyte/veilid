pub mod fixtures;
pub mod test_serialize_json;

use test_serialize_json::*;

#[expect(clippy::unused_async)]
pub async fn test_all() {
    test_serialize_json();
}
