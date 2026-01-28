use super::*;
use crate::tests::*;

fn add_mock_data(routing_table: &RoutingTable) {
    let pi = fix_peer_info(
        routing_table,
        fix_crypto_info_list(true),
        fix_crypto_info_list_secrets(),
    )
    .expect("should be valid");
    routing_table
        .register_node_with_peer_info(Arc::new(pi), false)
        .expect("should register");

    let _ = fix_peer_info(
        routing_table,
        fix_crypto_info_list(false),
        fix_crypto_info_list_secrets(),
    )
    .expect_err("should be invalid");

    let _ = fix_peer_info(
        routing_table,
        fix_crypto_info_list(true),
        SecretKeyGroup::new(),
    )
    .expect_err("should be missing a secret key");

    let pi3 =
        fix_unsigned_peer_info(routing_table, fix_crypto_info_list(true)).expect("should be valid");
    assert!(pi3.signatures().is_empty(), "should have no signatures");

    let _ = routing_table
        .register_node_with_peer_info(Arc::new(pi3.clone()), false)
        .expect_err("should fail with only no signatures");

    let _ = routing_table
        .register_node_with_peer_info(Arc::new(pi3), true)
        .expect("should succeed with allow_invalid");
}

pub async fn test_routingtable_buckets_round_trip() {
    let original_registry = mock_registry::init("a").await;
    let copy_registry = mock_registry::init("b").await;

    // Wrap to close lifetime of 'inner' which is borrowed here so terminate() can succeed
    // (it also .write() locks routing table inner)
    {
        let original = original_registry.routing_table();
        let copy = copy_registry.routing_table();

        add_mock_data(&original);

        let (serialized_bucket_map, all_entry_bytes) = original.serialized_buckets();

        RoutingTable::populate_routing_table_inner(
            &mut copy.inner.write(),
            serialized_bucket_map,
            all_entry_bytes,
        )
        .unwrap();

        let original_inner = &*original.inner.read();
        let copy_inner = &*copy.inner.read();

        let original_crypto_kinds: Vec<_> = original_inner.buckets.keys().clone().collect();
        let copy_crypto_kinds: Vec<_> = copy_inner.buckets.keys().clone().collect();

        assert_eq!(original_crypto_kinds.len(), copy_crypto_kinds.len());

        for crypto in original_crypto_kinds {
            // The same keys are present in the original and copy RoutingTables.
            let original_buckets = original_inner.buckets.get(crypto).unwrap();
            let copy_buckets = copy_inner.buckets.get(crypto).unwrap();

            // Recurse into RoutingTable.inner.buckets
            for (left_bucket, right_bucket) in original_buckets.iter().zip(copy_buckets.iter()) {
                // Recurse into RoutingTable.inner.buckets.entries
                for ((left_node_id, left_entry), (right_node_id, right_entry)) in
                    left_bucket.entries().zip(right_bucket.entries())
                {
                    assert_eq!(left_node_id, right_node_id);

                    let s = left_entry.with(original_inner, |_rti, e| serialize_json(e));
                    let s2 = right_entry.with(copy_inner, |_rti, e| serialize_json(e));

                    assert_eq!(s, s2);
                }
            }
        }
    }

    // Even if these are mocks, we should still practice good hygiene.
    mock_registry::terminate(original_registry).await;
    mock_registry::terminate(copy_registry).await;
}

pub async fn test_round_trip_peerinfo() {
    let registry = mock_registry::init("a").await;
    let routing_table = registry.routing_table();

    let pi = fix_peer_info(
        &routing_table,
        fix_crypto_info_list(true),
        fix_crypto_info_list_secrets(),
    )
    .expect("should be valid");

    let s = serialize_json(&pi);
    let pi2 = deserialize_json(&s).expect("Should deserialize");
    let s2 = serialize_json(&pi2);

    assert_eq!(pi, pi2);
    assert_eq!(s, s2);
}
