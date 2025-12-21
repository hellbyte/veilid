use super::*;

type PermReturnType = (Vec<usize>, SequenceOrderingSet);
type PermFunc<'t> = Box<dyn FnMut(&[usize]) -> Option<PermReturnType> + Send + 't>;

/// number of route permutations is the number of unique orderings
/// for a set of nodes, given that the first node is fixed
#[expect(dead_code)]
fn get_route_permutation_count(hop_count: usize) -> usize {
    if hop_count == 0 {
        unreachable!();
    }
    // a single node or two nodes is always fixed
    if hop_count == 1 || hop_count == 2 {
        return 1;
    }
    // more than two nodes has factorial permutation
    // hop_count = 3 -> 2! -> 2
    // hop_count = 4 -> 3! -> 6
    (3..hop_count).fold(2usize, |acc, x| acc * x)
}

/// get the route permutation at particular 'perm' index, starting at the 'start' index
/// for a set of 'hop_count' nodes. the first node is always fixed, and the maximum
/// number of permutations is given by get_route_permutation_count()
#[instrument(level = "trace", target = "route", skip_all)]
fn with_route_permutations(
    hop_count: usize,
    start: usize,
    f: &mut PermFunc,
) -> Option<PermReturnType> {
    if hop_count == 0 {
        unreachable!();
    }
    // initial permutation
    let mut permutation: Vec<usize> = Vec::with_capacity(hop_count);
    for n in 0..hop_count {
        permutation.push(start + n);
    }
    // if we have one hop or two, then there's only one permutation
    if hop_count == 1 || hop_count == 2 {
        return f(&permutation);
    }

    // heaps algorithm, but skipping the first element
    fn heaps_permutation(
        permutation: &mut [usize],
        size: usize,
        f: &mut PermFunc,
    ) -> Option<PermReturnType> {
        if size == 1 {
            return f(permutation);
        }

        for i in 0..size {
            let out = heaps_permutation(permutation, size - 1, f);
            if out.is_some() {
                return out;
            }
            if size % 2 == 1 {
                permutation.swap(1, size);
            } else {
                permutation.swap(1 + i, size);
            }
        }

        None
    }

    // recurse
    heaps_permutation(&mut permutation, hop_count - 1, f)
}

impl RouteSpecStore {
    /// Create a new route set
    /// Prefers nodes that are not currently in use by another route
    /// The route is not yet tested for its reachability
    /// Returns Err(VeilidAPIError::TryAgain) if no route could be allocated at this time
    /// Returns other errors on failure
    /// Returns Ok(route id string) on success
    #[instrument(level = "trace", target="route", skip(self), ret, err(level=Level::TRACE))]
    #[allow(clippy::too_many_arguments)]
    pub fn allocate_route(
        &self,
        crypto_kinds: &[CryptoKind],
        safety_spec: &SafetySpec,
        directions: DirectionSet,
        avoid_nodes: &[NodeId],
        automatic: bool,
    ) -> VeilidAPIResult<RouteId> {
        let inner = &mut *self.inner.lock();
        let routing_table = self.routing_table();
        let rti = &mut *routing_table.inner.write();

        self.allocate_route_inner(
            inner,
            rti,
            crypto_kinds,
            safety_spec,
            directions,
            avoid_nodes,
            automatic,
        )
    }

    /// Release an allocated route that is no longer in use
    #[instrument(level = "trace", target = "route", skip(self), ret)]
    pub(super) fn release_allocated_route(&self, id: RouteId) -> bool {
        let mut inner = self.inner.lock();
        let Some(rssd) = inner.content.remove_detail(&id) else {
            return false;
        };

        // Remove from hop cache
        let routing_table = self.routing_table();
        let rti = &*routing_table.inner.read();
        if !inner.cache.remove_from_cache(rti, id, &rssd) {
            panic!("hop cache should have contained cache key");
        }

        true
    }

    #[instrument(level = "trace", target="route", skip(self, inner, rti), ret, err(level=Level::TRACE))]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn allocate_route_inner(
        &self,
        inner: &mut RouteSpecStoreInner,
        rti: &mut RoutingTableInner,
        crypto_kinds: &[CryptoKind],
        safety_spec: &SafetySpec,
        directions: DirectionSet,
        avoid_nodes: &[NodeId],
        automatic: bool,
    ) -> VeilidAPIResult<RouteId> {
        if safety_spec.preferred_route.is_some() {
            apibail_generic!("safety_spec.preferred_route must be empty when allocating new route");
        }

        if safety_spec.hop_count < 1 {
            apibail_invalid_argument!(
                "Not allocating route less than one hop in length",
                "hop_count",
                safety_spec.hop_count
            );
        }

        if safety_spec.hop_count > self.get_max_route_hop_count() {
            apibail_invalid_argument!(
                "Not allocating route longer than max route hop count",
                "hop_count",
                safety_spec.hop_count
            );
        }

        // Get our peer info
        let Some(published_peer_info) = rti.get_published_peer_info(RoutingDomain::PublicInternet)
        else {
            apibail_try_again!(
                "unable to allocate route until we have a valid PublicInternet network class"
            );
        };

        let cur_ts = Timestamp::now();

        // Get list of all nodes, and sort them for selection
        let filter = self.make_route_allocation_entry_filter(
            rti,
            crypto_kinds,
            safety_spec,
            avoid_nodes,
            published_peer_info.clone(),
        );
        let filters = VecDeque::from([filter]);
        let compare = self.make_route_allocation_entry_sort(
            inner,
            cur_ts,
            safety_spec,
            published_peer_info.clone(),
        );
        let pre_sort_filter = self.make_route_allocation_pre_sort_filter(safety_spec);
        let transform = |_rti: &RoutingTableInner, entry: Option<Arc<BucketEntry>>| -> NodeRef {
            NodeRef::new(self.registry(), entry.unwrap())
        };

        // Pull the whole routing table in sorted order
        let nodes: Vec<NodeRef> = rti.find_peers_with_sort_and_filter(
            usize::MAX,
            cur_ts,
            filters,
            pre_sort_filter,
            compare,
            transform,
        );

        // If we couldn't find enough nodes, wait until we have more nodes in the routing table
        if nodes.len() < safety_spec.hop_count {
            apibail_try_again!("not enough nodes to construct route at this time");
        }

        // Now go through nodes and try to build a route we haven't seen yet
        let mut perm_func = self.make_route_allocation_permutation_function(
            inner,
            rti,
            &nodes,
            directions,
            safety_spec,
            published_peer_info.clone(),
        );

        let mut route_nodes: Vec<usize> = Vec::new();
        let mut orderings = SequenceOrderingSet::new();
        for start in 0..(nodes.len() - safety_spec.hop_count) {
            // Try the permutations available starting with 'start'
            if let Some((rn, ord)) =
                with_route_permutations(safety_spec.hop_count, start, &mut perm_func)
            {
                route_nodes = rn;
                orderings = ord;
                break;
            }
        }
        if route_nodes.is_empty() {
            apibail_try_again!("unable to find unique route at this time");
        }

        drop(perm_func);

        // Got a unique route, lets build the details, register it, and return it
        let hop_node_refs: Vec<NodeRef> = route_nodes.iter().map(|k| nodes[*k].clone()).collect();
        let mut route_set = BTreeMap::<PublicKey, RouteSpecDetail>::new();
        let crypto = self.crypto();
        for crypto_kind in crypto_kinds.iter().copied() {
            let vcrypto = crypto.get(crypto_kind).unwrap();
            let keypair = vcrypto.generate_keypair();
            let hops: Vec<NodeId> = route_nodes
                .iter()
                .map(|v| nodes[*v].locked(rti).node_ids().get(crypto_kind).unwrap())
                .collect();

            route_set.insert(
                keypair.key(),
                RouteSpecDetail {
                    secret_key: keypair.secret(),
                    hops,
                },
            );
        }

        let rssd = RouteSetSpecDetail::new(
            cur_ts,
            route_set,
            hop_node_refs,
            directions,
            safety_spec.stability,
            orderings,
            automatic,
        );

        // make id
        let id = self.generate_allocated_route_id(&rssd)?;

        // Add to cache
        inner.cache.add_to_cache(rti, &rssd);

        // Keep route in spec store
        inner.content.add_detail(id.clone(), rssd);

        Ok(id)
    }

    fn make_route_allocation_entry_filter<'t>(
        &self,
        rti: &mut RoutingTableInner,
        crypto_kinds: &'t [CryptoKind],
        safety_spec: &'t SafetySpec,
        avoid_nodes: &'t [NodeId],
        published_peer_info: Arc<PeerInfo>,
    ) -> RoutingTableEntryFilter<'t> {
        // Get our relay nodes if we have them
        let own_relay_atoms = rti
            .relays(RoutingDomain::PublicInternet)
            .iter()
            .map(|x| x.relay_node.entry().hash_atom())
            .collect::<HashSet<_>>();

        #[cfg(feature = "geolocation")]
        let country_code_denylist = self.config().network.privacy.country_code_denylist.clone();

        Box::new(
            move |rti: &RoutingTableInner,
                  entry: Option<Arc<BucketEntry>>,
                  _cur_ts: Timestamp|
                  -> bool {
                // Exclude our own node from routes
                let Some(entry) = entry else {
                    return false;
                };

                // Exclude our relay if we have one
                if own_relay_atoms.contains(&entry.clone().hash_atom()) {
                    return false;
                }

                // Process node info exclusions
                entry.with(rti, |_rti, e| {
                    // Exclude nodes that don't have our requested crypto kinds
                    let common_ck = e.common_crypto_kinds(crypto_kinds);
                    if common_ck.len() != crypto_kinds.len() {
                        return false;
                    }

                    // Exclude nodes we have specifically chosen to avoid
                    if e.node_ids().contains_any_from_slice(avoid_nodes) {
                        return false;
                    }

                    // Exclude nodes on our local network
                    if e.node_info(RoutingDomain::LocalNetwork).is_some() {
                        return false;
                    }

                    // Exclude nodes that have no publicinternet signednodeinfo
                    let Some(their_ni) = e.node_info(RoutingDomain::PublicInternet) else {
                        return false;
                    };

                    // Exclude nodes with no compatible dialinfo
                    if !their_ni.has_sequencing_matched_dial_info(safety_spec.sequencing) {
                        return false;
                    }

                    // Exclude nodes that have don't advertise route capability
                    if !their_ni.has_capability(VEILID_CAPABILITY_ROUTE) {
                        return false;
                    }

                    // Exclude nodes from denylisted countries
                    #[cfg(feature = "geolocation")]
                    if !country_code_denylist.is_empty() {
                        let geolocation_info =
                            their_ni.get_geolocation_info(RoutingDomain::PublicInternet);

                        // Since denylist is used, consider nodes with unknown countries to be automatically excluded
                        let Some(node_country_code) = geolocation_info.country_code() else {
                            veilid_log!(self
                                debug "allocate_route_inner: skipping node {:?} from unknown country",
                                e.best_node_id()
                            );
                            return false;
                        };
                        // The same thing applies to relays used by the node
                        // They must all be from a known country
                        let relay_country_codes: Option<Vec<CountryCode>> = geolocation_info.relay_country_codes().iter().cloned().collect();
                        let Some(relay_country_codes) = relay_country_codes else {
                            veilid_log!(self
                                debug "allocate_route_inner: skipping node {:?} using relay from unknown country",
                                e.best_node_id()
                            );
                            return false;
                        };

                        // Ensure that node is not excluded
                        if country_code_denylist.contains(&node_country_code)
                        {
                            veilid_log!(self
                                debug "allocate_route_inner: skipping node {:?} from excluded country {}",
                                e.best_node_id(),
                                node_country_code
                            );
                            return false;
                        }

                        // Ensure that node relays are not excluded
                        if let Some(cc) = relay_country_codes
                            .iter()
                            .filter(|cc| country_code_denylist.contains(cc))
                            .next()
                        {
                            veilid_log!(self
                                debug "allocate_route_inner: skipping node {:?} using relay from excluded country {}",
                                e.best_node_id(),
                                cc
                            );
                            return false;
                        }
                    }

                    // Filter out nodes that have our same public IP address
                    // Use whole ipv6 address so we don't filter out nodes in the same network
                    // These will be deprioritized in the sort later, though.
                    if published_peer_info.node_info().is_on_same_ipblock(their_ni, 128) {
                        return false;
                    }

                    // Relay check
                    for their_relay_info in their_ni.relay_info_list() {
                        // Exclude nodes whose relays we have chosen to avoid
                        if their_relay_info.node_ids().contains_any_from_slice(avoid_nodes) {
                            return false;
                        }
                    }
                    true
                })
            },
        )
    }

    fn make_route_allocation_entry_sort<'t>(
        &self,
        inner: &'t mut RouteSpecStoreInner,
        cur_ts: Timestamp,
        safety_spec: &'t SafetySpec,
        published_peer_info: Arc<PeerInfo>,
    ) -> RoutingTableEntrySort<'t> {
        let ip6_prefix_size = self.config().network.max_connections_per_ip6_prefix_size as usize;

        Box::new(
            move |_rti: &RoutingTableInner,
                  entry1: Option<Arc<BucketEntry>>,
                  entry2: Option<Arc<BucketEntry>>,
                  _cur_ts: Timestamp|
                  -> cmp::Ordering {
                // Our own node is filtered out, so it is safe to unwrap here
                let entry1 = entry1.as_ref().unwrap().clone();
                let entry2 = entry2.as_ref().unwrap().clone();
                let entry1_node_ids = entry1.with_inner(|e| e.node_ids());
                let entry2_node_ids = entry2.with_inner(|e| e.node_ids());
                let entry1_node_info = entry1
                    .with_inner(|e| e.node_info(RoutingDomain::PublicInternet).cloned().unwrap());
                let entry2_node_info = entry2
                    .with_inner(|e| e.node_info(RoutingDomain::PublicInternet).cloned().unwrap());

                // deprioritize nodes that we have already used as end points
                let e1_used_end = inner.cache.get_used_end_node_count(&entry1_node_ids);
                let e2_used_end = inner.cache.get_used_end_node_count(&entry2_node_ids);
                let cmp_used_end = e1_used_end.cmp(&e2_used_end);
                if !matches!(cmp_used_end, cmp::Ordering::Equal) {
                    return cmp_used_end;
                }

                // deprioritize nodes we have used already anywhere
                let e1_used = inner.cache.get_used_node_count(&entry1_node_ids);
                let e2_used = inner.cache.get_used_node_count(&entry2_node_ids);
                let cmp_used = e1_used.cmp(&e2_used);
                if !matches!(cmp_used, cmp::Ordering::Equal) {
                    return cmp_used;
                }

                // deprioritize nodes that are on our own ipv6 network
                // this check also checks if the ipv4 address is the same but we filtered that out already
                let e1_same_ipblock = published_peer_info
                    .node_info()
                    .is_any_node_on_same_ipblock(&entry1_node_info, ip6_prefix_size);
                let e2_same_ipblock = published_peer_info
                    .node_info()
                    .is_any_node_on_same_ipblock(&entry2_node_info, ip6_prefix_size);
                let cmp_same_ipblock = e1_same_ipblock.cmp(&e2_same_ipblock);
                if !matches!(cmp_same_ipblock, cmp::Ordering::Equal) {
                    return cmp_same_ipblock;
                }

                // apply sequencing preference
                // ensureordered will be taken care of by filter
                // and nopreference doesn't care
                if matches!(safety_spec.sequencing, Sequencing::PreferOrdered) {
                    let cmp_seq = entry1.with_inner(|e1| {
                        entry2.with_inner(|e2| {
                            let e1_can_do_ordered = e1
                                .node_info(RoutingDomain::PublicInternet)
                                .map(|ni| {
                                    ni.has_sequencing_matched_dial_info(safety_spec.sequencing)
                                })
                                .unwrap_or(false);
                            let e2_can_do_ordered = e2
                                .node_info(RoutingDomain::PublicInternet)
                                .map(|ni| {
                                    ni.has_sequencing_matched_dial_info(safety_spec.sequencing)
                                })
                                .unwrap_or(false);
                            // Reverse this comparison because ordered is preferable (less)
                            e2_can_do_ordered.cmp(&e1_can_do_ordered)
                        })
                    });
                    if !matches!(cmp_seq, cmp::Ordering::Equal) {
                        return cmp_seq;
                    }
                }

                // apply stability preference
                // always prioritize reliable nodes, but sort by oldest or fastest
                entry1.with_inner(|e1| {
                    entry2.with_inner(|e2| match safety_spec.stability {
                        Stability::LowLatency => {
                            BucketEntryInner::cmp_fastest_reliable(cur_ts, e1, e2, |ls| ls.tm90)
                        }
                        Stability::Reliable => {
                            BucketEntryInner::cmp_oldest_reliable(cur_ts, e1, e2)
                        }
                    })
                })
            },
        ) as RoutingTableEntrySort
    }

    fn make_route_allocation_pre_sort_filter<'t>(
        &self,
        safety_spec: &'t SafetySpec,
    ) -> RoutingTableEntryPreSortFilter<'t> {
        Box::new(
            move |_rti: &RoutingTableInner,
                  all_entries: &mut Vec<Option<Arc<BucketEntry>>>,
                  _cur_ts: Timestamp| {
                // Remove the slowest 20% of the entries from consideration
                let mut sorted_entries = all_entries.clone();
                sorted_entries.sort_by(|entry1, entry2| {
                    let entry1 = entry1.as_ref().unwrap().clone();
                    let entry2 = entry2.as_ref().unwrap().clone();

                    entry1.with_inner(|e1| {
                        entry2.with_inner(|e2| BucketEntryInner::cmp_fastest(e1, e2, |ls| ls.tm90))
                    })
                });

                let reduce = sorted_entries.len() / 5;

                sorted_entries.truncate((sorted_entries.len() - reduce).max(safety_spec.hop_count));

                // Make set of nodes to keep
                let keepers = sorted_entries
                    .iter()
                    .map(|e| e.as_ref().unwrap().clone().hash_atom())
                    .collect::<HashSet<_>>();

                // Retain only entries from the keepers set
                // This preserves the order of the entries while removing the slow ones
                all_entries.retain(|x| {
                    let atom = x.as_ref().unwrap().clone().hash_atom();
                    keepers.contains(&atom)
                })
            },
        ) as RoutingTableEntryPreSortFilter
    }

    // Get the hop cache key for a particular route permutation
    // uses the same algorithm as RouteSetSpecDetail::make_cache_key
    fn route_permutation_to_hop_cache(
        rti: &RoutingTableInner,
        nodes: &[NodeRef],
        perm: &[usize],
    ) -> Option<Vec<u8>> {
        let mut cachelen = 0usize;
        let mut nodebytes = Vec::<BareNodeId>::with_capacity(perm.len());
        for n in perm {
            let b = nodes[*n].locked(rti).best_node_id()?.value();
            cachelen += b.len();
            nodebytes.push(b);
        }
        let mut cache: Vec<u8> = Vec::with_capacity(cachelen);
        for b in nodebytes {
            cache.extend_from_slice(&b);
        }
        Some(cache)
    }

    fn make_route_allocation_permutation_function<'t>(
        &self,
        inner: &'t mut RouteSpecStoreInner,
        rti: &'t mut RoutingTableInner,
        nodes: &'t [NodeRef],
        directions: DirectionSet,
        safety_spec: &'t SafetySpec,
        published_peer_info: Arc<PeerInfo>,
    ) -> PermFunc<'t> {
        // Get peer info for everything
        let nodes_pi: Vec<Arc<PeerInfo>> = nodes
            .iter()
            .map(|nr| {
                nr.locked(rti)
                    .get_peer_info(RoutingDomain::PublicInternet)
                    .unwrap()
            })
            .collect();

        Box::new(move |permutation: &[usize]| {
            let cache_key = Self::route_permutation_to_hop_cache(rti, nodes, permutation)?;

            // Skip routes we have already seen
            if inner.cache.contains_route(&cache_key) {
                return None;
            }

            // Ensure the route doesn't contain both a node and its relay
            let mut seen_nodes: HashSet<NodeId> = HashSet::new();
            for n in permutation {
                let node = nodes.get(*n).unwrap();
                for nid in node.locked(rti).node_ids().iter() {
                    if !seen_nodes.insert(nid.clone()) {
                        // Already seen this node, should not be in the route twice
                        return None;
                    }
                }
                for rids in node
                    .locked_mut(rti)
                    .relay_ids(RoutingDomain::PublicInternet)
                {
                    for rid in rids.iter() {
                        if !seen_nodes.insert(rid.clone()) {
                            // Already seen this node, should not be in the route twice
                            return None;
                        }
                    }
                }
            }

            // Ensure this route is viable by checking that each node can contact the next one
            let mut orderings = SequenceOrderingSet::all();
            if directions.contains(Direction::Out) {
                let mut previous_node = published_peer_info.clone();
                let mut reachable = true;
                for n in permutation {
                    let current_node = nodes_pi.get(*n).cloned().unwrap();
                    let cm = rti.get_contact_method(
                        RoutingDomain::PublicInternet,
                        previous_node.clone(),
                        current_node.clone(),
                        DialInfoFilter::all(),
                        safety_spec.sequencing,
                        None,
                    );
                    if matches!(cm, ContactMethod::Unreachable) {
                        reachable = false;
                        break;
                    }

                    // Check if we can do each ordering strictly
                    for ordering in orderings {
                        let cm = rti.get_contact_method(
                            RoutingDomain::PublicInternet,
                            previous_node.clone(),
                            current_node.clone(),
                            DialInfoFilter::all(),
                            ordering.strict_sequencing(),
                            None,
                        );
                        if matches!(cm, ContactMethod::Unreachable) {
                            orderings.remove(ordering);
                        }
                    }

                    previous_node = current_node;
                }
                if !reachable {
                    return None;
                }
            }
            if directions.contains(Direction::In) {
                let mut next_node = published_peer_info.clone();
                let mut reachable = true;
                for n in permutation.iter().rev() {
                    let current_node = nodes_pi.get(*n).cloned().unwrap();
                    let cm = rti.get_contact_method(
                        RoutingDomain::PublicInternet,
                        next_node.clone(),
                        current_node.clone(),
                        DialInfoFilter::all(),
                        safety_spec.sequencing,
                        None,
                    );
                    if matches!(cm, ContactMethod::Unreachable) {
                        reachable = false;
                        break;
                    }

                    // Check if we can do each ordering strictly
                    for ordering in orderings {
                        let cm = rti.get_contact_method(
                            RoutingDomain::PublicInternet,
                            next_node.clone(),
                            current_node.clone(),
                            DialInfoFilter::all(),
                            ordering.strict_sequencing(),
                            None,
                        );
                        if matches!(cm, ContactMethod::Unreachable) {
                            orderings.remove(ordering);
                        }
                    }
                    next_node = current_node;
                }
                if !reachable {
                    return None;
                }
            }
            // Keep this route
            let route_nodes = permutation.to_vec();
            Some((route_nodes, orderings))
        }) as PermFunc
    }

    /// Generate RouteId from typed key set of route public keys
    fn generate_allocated_route_id(&self, rssd: &RouteSetSpecDetail) -> VeilidAPIResult<RouteId> {
        let route_set_keys = rssd.get_route_set_keys();
        let crypto = self.crypto();

        let pkbyteslen = route_set_keys
            .iter()
            .fold(0, |acc, x| acc + x.ref_value().len());
        let mut pkbytes = Vec::with_capacity(pkbyteslen);
        let mut best_kind: Option<CryptoKind> = None;
        for tk in route_set_keys.iter() {
            if best_kind.is_none()
                || compare_crypto_kind(&tk.kind(), best_kind.as_ref().unwrap())
                    == cmp::Ordering::Less
            {
                best_kind = Some(tk.kind());
            }
            pkbytes.extend_from_slice(tk.ref_value());
        }
        let Some(best_kind) = best_kind else {
            apibail_internal!("no compatible crypto kinds in route");
        };
        let vcrypto = crypto.get(best_kind).unwrap();

        Ok(RouteId::new(
            vcrypto.kind(),
            BareRouteId::new(vcrypto.generate_hash(&pkbytes).ref_value()),
        ))
    }
}
