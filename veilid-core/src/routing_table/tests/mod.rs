pub mod fixtures;
pub mod test_serialize_routing_table;
pub mod test_signed_node_info;

pub async fn test_all() {
    test_serialize_routing_table::test_routingtable_buckets_round_trip().await;
    test_serialize_routing_table::test_round_trip_peerinfo().await;
    test_signed_node_info::test_signed_node_info().await;
}
