use super::*;
use crate::{routing_table::*, RegisteredComponents, VALID_CRYPTO_KINDS};

fn make_mock_typed_node_id(kind: CryptoKind, idx: u8) -> TypedNodeId {
    TypedNodeId::new(
        kind,
        NodeId::new([
            idx, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]),
    )
}

fn make_mock_typed_node_id_group(valid_kinds: bool, unknown: bool) -> TypedNodeIdGroup {
    let mut tks = TypedNodeIdGroup::new();
    if valid_kinds {
        VALID_CRYPTO_KINDS.iter().for_each(|k| {
            tks.add(make_mock_typed_node_id(*k, 0));
        });
    }
    if unknown {
        tks.add(make_mock_typed_node_id(CryptoKind([1, 2, 3, 4]), 0));
    }
    tks
}

fn make_mock_peer_info(node_ids: TypedNodeIdGroup) -> EyreResult<PeerInfo> {
    PeerInfo::new(
        RoutingDomain::PublicInternet,
        node_ids,
        SignedNodeInfo::Direct(SignedDirectNodeInfo::new(
            NodeInfo::new(
                NetworkClass::OutboundOnly,
                ProtocolTypeSet::new(),
                AddressTypeSet::new(),
                vec![0],
                vec![CRYPTO_KIND_VLD0],
                PUBLIC_INTERNET_CAPABILITIES.to_vec(),
                vec![],
            ),
            Timestamp::new(0),
            Vec::new(),
        )),
    )
}

fn add_mock_data(routing_table: &VeilidComponentGuard<'_, RoutingTable>) {
    let pi =
        make_mock_peer_info(make_mock_typed_node_id_group(true, false)).expect("should be valid");
    routing_table
        .register_node_with_peer_info(Arc::new(pi), true)
        .expect("should register");
    let pi2 =
        make_mock_peer_info(make_mock_typed_node_id_group(true, true)).expect("should be valid");
    routing_table
        .register_node_with_peer_info(Arc::new(pi2), true)
        .expect("should register");

    let _ = make_mock_peer_info(make_mock_typed_node_id_group(false, false))
        .expect_err("should fail with no node ids");

    let pi3 =
        make_mock_peer_info(make_mock_typed_node_id_group(false, true)).expect("should be valid");

    let _ = routing_table
        .register_node_with_peer_info(Arc::new(pi3), true)
        .expect_err("should fail with only unsupported node ids");
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

pub fn test_round_trip_peerinfo() {
    let pi =
        make_mock_peer_info(make_mock_typed_node_id_group(true, true)).expect("should be valid");

    let s = serialize_json(&pi);
    let pi2 = deserialize_json(&s).expect("Should deserialize");
    let s2 = serialize_json(&pi2);

    assert_eq!(pi, pi2);
    assert_eq!(s, s2);
}

pub async fn test_all() {
    test_routingtable_buckets_round_trip().await;
    test_round_trip_peerinfo();
}
