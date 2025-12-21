use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteSpecDetail {
    /// Secret key
    pub secret_key: SecretKey,
    /// Route hop node ids
    pub hops: Vec<NodeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteSetSpecDetail {
    /// Routes in the set
    route_set: BTreeMap<PublicKey, RouteSpecDetail>,
    /// Route noderefs
    #[serde(skip)]
    hop_node_refs: Vec<NodeRef>,
    /// Published private route, do not reuse for ephemeral routes
    /// Not serialized because all routes should be re-published when restarting
    #[serde(skip)]
    published: bool,
    /// Directions this route is guaranteed to work in
    directions: DirectionSet,
    /// Stability preference (prefer reliable nodes over faster)
    stability: Stability,
    /// Sequencing capability (connection oriented protocols vs datagram)
    orderings: SequenceOrderingSet,
    /// Stats
    stats: RouteStats,
    /// Automatically allocated route vs manually allocated route
    automatic: bool,
}

impl fmt::Display for RouteSetSpecDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "count={}, stability={:?} dirs={:?} auto={:?} pub={:?} latency={} transfer={} last_valid={} hops=[{}]",
            self.hop_count(),
            self.get_stability(),
            self.get_directions(),
            self.is_automatic(),
            self.is_published(),
            self.get_stats().latency_stats(),
            self.get_stats().transfer_stats(),
            self.get_stats().last_known_valid_ts.map(|ts| ts.to_string()).unwrap_or_else(|| "None".to_string()),
            self.hop_node_refs().iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
        )
    }
}

impl RouteSetSpecDetail {
    pub fn new(
        cur_ts: Timestamp,
        route_set: BTreeMap<PublicKey, RouteSpecDetail>,
        hop_node_refs: Vec<NodeRef>,
        directions: DirectionSet,
        stability: Stability,
        orderings: SequenceOrderingSet,
        automatic: bool,
    ) -> Self {
        Self {
            route_set,
            hop_node_refs,
            published: false,
            directions,
            stability,
            orderings,
            stats: RouteStats::new(cur_ts),
            automatic,
        }
    }
    #[expect(dead_code)]
    pub fn len(&self) -> usize {
        self.route_set.len()
    }
    #[expect(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.route_set.is_empty()
    }
    pub fn get_route_by_key(&self, key: &PublicKey) -> Option<&RouteSpecDetail> {
        self.route_set.get(key)
    }
    pub fn get_route_set_keys(&self) -> PublicKeyGroup {
        let mut tks = PublicKeyGroup::new();
        for k in self.route_set.keys() {
            tks.add(k.clone());
        }
        tks
    }
    pub fn get_best_route_set_key(&self) -> Option<PublicKey> {
        self.get_route_set_keys().first().cloned()
    }
    pub fn set_hop_node_refs(&mut self, node_refs: Vec<NodeRef>) {
        self.hop_node_refs = node_refs;
    }
    pub fn iter_route_set(
        &self,
    ) -> alloc::collections::btree_map::Iter<'_, PublicKey, RouteSpecDetail> {
        self.route_set.iter()
    }
    #[expect(dead_code)]
    pub fn iter_route_set_mut(
        &mut self,
    ) -> alloc::collections::btree_map::IterMut<'_, PublicKey, RouteSpecDetail> {
        self.route_set.iter_mut()
    }
    #[expect(dead_code)]
    pub fn remove_route(&mut self, key: &PublicKey) {
        self.route_set.remove(key);
    }
    pub fn get_stats(&self) -> &RouteStats {
        &self.stats
    }
    pub fn get_stats_mut(&mut self) -> &mut RouteStats {
        &mut self.stats
    }
    pub fn is_published(&self) -> bool {
        self.published
    }
    pub fn set_published(&mut self, published: bool) {
        self.published = published;
    }
    pub fn hop_count(&self) -> usize {
        self.hop_node_refs.len()
    }
    pub fn hop_node_refs(&self) -> Vec<NodeRef> {
        self.hop_node_refs.clone()
    }
    pub fn hop_node_ref(&self, idx: usize) -> Option<NodeRef> {
        self.hop_node_refs.get(idx).cloned()
    }
    pub fn get_stability(&self) -> Stability {
        self.stability
    }
    pub fn get_directions(&self) -> DirectionSet {
        self.directions
    }
    pub fn is_sequencing_match(&self, sequencing: Sequencing) -> bool {
        for ordering in self.orderings.iter() {
            if sequencing.matches_ordering(ordering) {
                return true;
            }
        }
        false
    }
    pub fn contains_nodes(&self, nodes: &[NodeId]) -> bool {
        for tk in nodes {
            for rsd in self.route_set.values() {
                if rsd.hops.contains(tk) {
                    return true;
                }
            }
        }
        false
    }
    pub fn is_automatic(&self) -> bool {
        self.automatic
    }
    /// Generate a key for the cache that can be used to uniquely identify this route's contents
    pub fn make_cache_key(&self, rti: &RoutingTableInner) -> Vec<u8> {
        let hops = &self.hop_node_refs;
        let mut cache: Vec<u8> = Vec::with_capacity(hops.len() * 32); // xxx hack: this code is going away soon anyway
        for hop in hops {
            if let Some(b) = hop.locked(rti).best_node_id() {
                cache.extend_from_slice(b.ref_value());
            }
        }
        cache
    }
}
