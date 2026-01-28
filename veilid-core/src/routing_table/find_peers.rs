use super::*;

impl RoutingTable {
    /// Utility to find the closest nodes to a particular hash coordinate, preferring reliable nodes first,
    /// including possibly our own node and nodes further away from the key than our own,
    /// returning their peer info
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn find_preferred_closest_peers(
        &self,
        routing_domain: RoutingDomain,
        hash_coordinate: HashCoordinate,
        capabilities: &[VeilidCapability],
    ) -> NetworkResult<Vec<Arc<PeerInfo>>> {
        if Crypto::validate_crypto_kind(hash_coordinate.kind()).is_err() {
            return NetworkResult::invalid_message("invalid crypto kind");
        }

        let opt_published_peer_info = self.get_published_peer_info(routing_domain);
        let include_self = opt_published_peer_info
            .as_ref()
            .map(|x| x.node_info().has_all_capabilities(capabilities))
            .unwrap_or_default();

        // find N nodes closest to the target node in our routing table
        let filter = Box::new(
            |_rti: &RoutingTableInner, opt_entry: Option<Arc<BucketEntry>>, _cur_ts: Timestamp| {
                // Ensure only things that are valid in the chosen routing domain,
                // and with matching capabilities are returned
                match opt_entry {
                    Some(entry) => entry.with_inner(|e| {
                        e.get_peer_info(routing_domain).is_some()
                            && e.has_all_capabilities(routing_domain, capabilities)
                    }),
                    None => include_self,
                }
            },
        ) as RoutingTableEntryFilter;
        let filters = VecDeque::from([filter]);

        let node_count = self.config().network.dht.max_find_node_count as usize;

        let closest_nodes = match self.find_preferred_closest_nodes(
            node_count,
            hash_coordinate.clone(),
            filters,
            // transform
            |_rti, opt_entry| match opt_entry {
                Some(entry) => {
                    entry.with_inner(|e| e.get_peer_info(routing_domain).unwrap_or_log())
                }
                None => opt_published_peer_info.clone().unwrap_or_log(),
            },
        ) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "failed to find closest nodes for key {}: {}",
                    hash_coordinate, e
                );
                return NetworkResult::invalid_message("failed to find closest nodes for key");
            }
        };

        NetworkResult::value(closest_nodes)
    }

    /// Utility to find nodes that are closer to a key than our own node,
    /// returning only reliable nodes, and returning their peer info
    /// Can filter based on a particular set of capabilities
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab", skip_all, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn find_reliable_peers_closer_to_key(
        &self,
        routing_domain: RoutingDomain,
        hash_coordinate: HashCoordinate,
        required_capabilities: Vec<VeilidCapability>,
    ) -> NetworkResult<Vec<Arc<PeerInfo>>> {
        // add node information for the requesting node to our routing table
        let crypto_kind = hash_coordinate.kind();
        let own_node_id = self.node_id(crypto_kind);

        // find N nodes closest to the target node in our routing table
        // ensure the nodes returned are only the ones closer to the target node than ourself
        let own_distance = own_node_id.to_hash_coordinate().distance(&hash_coordinate);

        let hash_coordinate2 = hash_coordinate.clone();
        let filter = Box::new(
            move |_rti: &RoutingTableInner,
                  opt_entry: Option<Arc<BucketEntry>>,
                  cur_ts: Timestamp| {
                // Exclude our own node
                let Some(entry) = opt_entry else {
                    return false;
                };
                // Ensure only things that have a minimum set of capabilities are returned
                entry.with_inner(|e| {
                    // Ensure only things that are valid in the chosen routing domain,
                    // and with matching capabilities are returned
                    if e.get_peer_info(routing_domain).is_none()
                        || !e.has_all_capabilities(routing_domain, &required_capabilities)
                    {
                        return false;
                    }

                    if e.check_unreliable(cur_ts).is_some() {
                        return false;
                    }

                    // Ensure things further from the key than our own node are not included
                    let Some(entry_node_id) = e.node_ids().get(crypto_kind) else {
                        return false;
                    };
                    let entry_distance = entry_node_id
                        .to_hash_coordinate()
                        .distance(&hash_coordinate2);
                    if entry_distance >= own_distance {
                        return false;
                    }
                    true
                })
            },
        ) as RoutingTableEntryFilter;
        let filters = VecDeque::from([filter]);

        let node_count = self.config().network.dht.max_find_node_count as usize;

        //
        let closest_nodes = match self.find_preferred_closest_nodes(
            node_count,
            hash_coordinate.clone(),
            filters,
            // transform
            |rti, entry| {
                entry.unwrap_or_log().with(rti, |_rti, e| {
                    e.get_peer_info(routing_domain).unwrap_or_log()
                })
            },
        ) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "failed to find closest nodes for key {}: {}",
                    hash_coordinate, e
                );
                return NetworkResult::invalid_message("failed to find closest nodes for key");
            }
        };

        // Validate peers returned are, in fact, closer to the key than the node we sent this to
        // This same test is used on the other side so we vet things here
        let valid = match self.verify_peers_closer(
            own_node_id.to_hash_coordinate(),
            hash_coordinate.clone(),
            &closest_nodes,
        ) {
            Ok(v) => v,
            Err(e) => {
                panic!("missing cryptosystem in peers node ids: {}", e);
            }
        };
        if !valid {
            error!(
                "non-closer peers returned: own_node_id={:#?} key={:#?} closest_nodes={:#?}",
                own_node_id, hash_coordinate, closest_nodes
            );
        }

        NetworkResult::value(closest_nodes)
    }

    /// Determine if set of peers is closer to key_near than key_far is to key_near
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "rtab", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub fn verify_peers_closer(
        &self,
        hash_coordinate_far: HashCoordinate,
        hash_coordinate_near: HashCoordinate,
        peers: &[Arc<PeerInfo>],
    ) -> EyreResult<bool> {
        if hash_coordinate_far.kind() != hash_coordinate_near.kind() {
            bail!("keys all need the same cryptosystem");
        }

        let mut closer = true;
        let d_far = hash_coordinate_far.distance(&hash_coordinate_near);
        for peer in peers {
            let Some(key_peer) = peer.node_ids().get(hash_coordinate_far.kind()) else {
                bail!("peers need to have a key with the same cryptosystem");
            };
            let d_near = hash_coordinate_near.distance(&key_peer.to_hash_coordinate());
            if d_far < d_near {
                let warning = format!(
                    r#"peer: {}
near (key): {}
far (self): {}
    d_near: {}
     d_far: {}
       cmp: {:?}"#,
                    key_peer,
                    hash_coordinate_near,
                    hash_coordinate_far,
                    d_near,
                    d_far,
                    d_near.cmp(&d_far)
                );
                veilid_log!(self warn "{}", warning);
                closer = false;
                break;
            }
        }

        Ok(closer)
    }
}
