use super::*;

pub const NODE_CONTACT_METHOD_CACHE_SIZE: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct NodeContactMethodCacheKey {
    /// The node ids associated with this target
    pub node_ids: NodeIdGroup,
    /// The timestamp of our own -current- peer info
    pub own_node_info_ts: Timestamp,
    /// The timestamp of the target's published peer info
    pub target_node_info_ts: Timestamp,
    /// The node ref filter used to connect to the target
    pub target_node_ref_filter: NodeRefFilter,
    /// The sequencing requirement for connections to the target
    pub target_node_ref_sequencing: Sequencing,
    /// The ordering of dial info failures from least recent to most recent
    pub dial_info_failures: Vec<DialInfo>,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct HitMissStats {
    pub hit: usize,
    pub miss: usize,
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum ContactMethodKind {
    Unreachable,
    Existing,
    Direct,
    SignalReverse,
    SignalHolePunch,
    InboundRelay,
    OutboundRelay,
}
impl From<Option<&NodeContactMethodKind>> for ContactMethodKind {
    fn from(value: Option<&NodeContactMethodKind>) -> Self {
        match value {
            None => ContactMethodKind::Unreachable,
            Some(NodeContactMethodKind::Existing) => ContactMethodKind::Existing,
            Some(NodeContactMethodKind::Direct { target_di: _ }) => ContactMethodKind::Direct,
            Some(NodeContactMethodKind::SignalReverse {
                relay_nr: _,
                target_nr: _,
            }) => ContactMethodKind::SignalReverse,
            Some(NodeContactMethodKind::SignalHolePunch {
                relay_nr: _,
                target_nr: _,
            }) => ContactMethodKind::SignalHolePunch,
            Some(NodeContactMethodKind::InboundRelay { relay_nr: _ }) => {
                ContactMethodKind::InboundRelay
            }
            Some(NodeContactMethodKind::OutboundRelay { relay_nr: _ }) => {
                ContactMethodKind::OutboundRelay
            }
        }
    }
}

impl HitMissStats {
    pub fn percentage(&self) -> f32 {
        (self.hit as f32 * 100.0f32) / ((self.hit + self.miss) as f32)
    }
}

impl fmt::Display for HitMissStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} {:.2}%",
            self.hit,
            self.hit + self.miss,
            self.percentage()
        )
    }
}

pub struct NodeContactMethodCache {
    cache: LruCache<NodeContactMethodCacheKey, Option<NodeContactMethodKind>>,

    // Statistics for cache hits/misses
    cache_stats: HitMissStats,

    // Recorded stats for contact method success
    contact_method_kind_stats: HashMap<ContactMethodKind, HitMissStats>,
}

impl fmt::Debug for NodeContactMethodCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeContactMethodCache")
            //.field("cache", &self.cache)
            .field("cache_stats", &self.cache_stats)
            .field("contact_method_kind_stats", &self.contact_method_kind_stats)
            .finish()
    }
}

impl NodeContactMethodCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NODE_CONTACT_METHOD_CACHE_SIZE),
            cache_stats: HitMissStats::default(),
            contact_method_kind_stats: HashMap::new(),
        }
    }
    pub fn insert(
        &mut self,
        ncm_key: NodeContactMethodCacheKey,
        opt_ncm_kind: Option<NodeContactMethodKind>,
    ) {
        // Cache this
        self.cache.insert(ncm_key, opt_ncm_kind);
    }

    pub fn get(
        &mut self,
        ncm_key: &NodeContactMethodCacheKey,
    ) -> Option<Option<NodeContactMethodKind>> {
        if let Some(opt_ncm_kind) = self.cache.get(ncm_key) {
            self.cache_stats.hit += 1;

            return Some(opt_ncm_kind.clone());
        }
        // Record miss
        self.cache_stats.miss += 1;
        None
    }

    pub fn record_contact_method_success(&mut self, ncm_kind: Option<&NodeContactMethodKind>) {
        let cmk = ContactMethodKind::from(ncm_kind);
        self.contact_method_kind_stats.entry(cmk).or_default().hit += 1;
    }
    pub fn record_contact_method_failure(&mut self, ncm_kind: Option<&NodeContactMethodKind>) {
        let cmk = ContactMethodKind::from(ncm_kind);
        self.contact_method_kind_stats.entry(cmk).or_default().miss += 1;
    }

    pub fn debug(&self) -> String {
        let mut out = format!(
            "Cache size: {}\nCache hits: {}\nContact methods:\n",
            self.cache.len(),
            self.cache_stats
        );
        let mut sorted_kinds: Vec<_> = self.contact_method_kind_stats.keys().collect();
        sorted_kinds.sort();
        for kind in sorted_kinds {
            let kindstats = self.contact_method_kind_stats.get(kind).unwrap();
            out += &format!("  {:?}: {}\n", kind, kindstats);
        }
        out
    }
}
