pub mod test_bootstrap;
pub mod test_connection_table;

use super::*;

pub async fn test_all() {
    test_bootstrap::test_bootstrap_v0().await;
    test_bootstrap::test_bootstrap_v1().await;
    test_connection_table::test_add_get_remove().await;
}
