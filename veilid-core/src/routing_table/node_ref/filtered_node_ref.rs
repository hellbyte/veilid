use super::*;

impl_veilid_log_facility!("rtab");

pub(crate) struct FilteredNodeRef {
    registry: VeilidComponentRegistry,
    entry: Arc<BucketEntry>,
    filter: NodeRefFilter,
    sequencing: Sequencing,
    #[cfg(feature = "tracking")]
    track_id: usize,
}

impl_veilid_component_accessors!(FilteredNodeRef);

impl FilteredNodeRef {
    pub fn new(
        registry: VeilidComponentRegistry,
        entry: Arc<BucketEntry>,
        filter: NodeRefFilter,
        sequencing: Sequencing,
    ) -> Self {
        entry.ref_count.fetch_add(1u32, Ordering::AcqRel);

        Self {
            registry,
            entry,
            filter,
            sequencing,
            #[cfg(feature = "tracking")]
            track_id: entry.track(),
        }
    }

    pub fn unfiltered(&self) -> NodeRef {
        NodeRef::new(self.registry(), self.entry.clone())
    }

    pub fn merge_filter_clone(&self, filter: NodeRefFilter) -> FilteredNodeRef {
        let mut out = self.clone();
        out.merge_filter(filter);
        out
    }

    pub fn with_sequencing(&self, sequencing: Sequencing) -> FilteredNodeRef {
        FilteredNodeRef::new(
            self.registry.clone(),
            self.entry.clone(),
            self.filter(),
            sequencing,
        )
    }
    pub fn with_routing_domain<R: Into<RoutingDomainSet>>(
        &self,
        routing_domain_set: R,
    ) -> FilteredNodeRef {
        FilteredNodeRef::new(
            self.registry.clone(),
            self.entry.clone(),
            self.filter()
                .with_routing_domain_set(routing_domain_set.into()),
            self.sequencing(),
        )
    }

    pub fn locked<'a>(&self, rti: &'a RoutingTableInner) -> LockedFilteredNodeRef<'a> {
        LockedFilteredNodeRef::new(rti, self.clone())
    }

    pub fn set_filter(&mut self, filter: NodeRefFilter) {
        self.filter = filter
    }

    pub fn merge_filter(&mut self, filter: NodeRefFilter) {
        self.filter = self.filter.filtered(filter);
    }

    pub fn set_sequencing(&mut self, sequencing: Sequencing) {
        self.sequencing = sequencing;
    }

    pub fn parse<S: AsRef<str>>(
        routing_table: &RoutingTable,
        s: S,
    ) -> VeilidAPIResult<Option<Self>> {
        let text = s.as_ref();

        // NodeRefFilter mods
        let (text, mods) = text
            .split_once('/')
            .map(|x| (x.0, Some(x.1)))
            .unwrap_or((text, None));
        let filter = match mods {
            Some(mods) => Some(NodeRefFilter::from_str(mods)?),
            None => None,
        };

        // Sequencing
        let (text, seq) = if let Some((first, second)) = text.split_once('+') {
            let seq = Sequencing::from_str(second)?;
            (first, Some(seq))
        } else {
            (text, None)
        };

        // NodeId
        if text.is_empty() {
            apibail_parse_error!(
                "FilteredNodeRef::parse missing node id",
                s.as_ref().to_string()
            );
        }
        let nr = if let Ok(key) = BareNodeId::from_str(text) {
            routing_table
                .lookup_any_node_ref(key)
                .map_err(VeilidAPIError::internal)?
        } else if let Ok(node_id) = NodeId::from_str(text) {
            routing_table
                .lookup_node_ref(node_id)
                .map_err(VeilidAPIError::internal)?
        } else {
            apibail_parse_error!(
                "FilteredNodeRef::parse invalid node id",
                s.as_ref().to_string()
            );
        };
        let Some(nr) = nr else { return Ok(None) };

        // Filter the noderef
        let nrf = if let Some(filter) = filter {
            nr.custom_filtered(filter)
        } else {
            nr.default_filtered()
        };
        let opt_nrf = if let Some(seq) = seq {
            Some(nrf.with_sequencing(seq))
        } else {
            Some(nrf)
        };
        Ok(opt_nrf)
    }
}

impl NodeRefAccessorsTrait for FilteredNodeRef {
    fn entry(&self) -> Arc<BucketEntry> {
        self.entry.clone()
    }

    fn sequencing(&self) -> Sequencing {
        self.sequencing
    }

    fn routing_domain_set(&self) -> RoutingDomainSet {
        self.filter.routing_domain_set
    }

    fn filter(&self) -> NodeRefFilter {
        self.filter
    }

    fn take_filter(&mut self) -> NodeRefFilter {
        let f = self.filter;
        self.filter = NodeRefFilter::new();
        f
    }

    fn dial_info_filter(&self) -> DialInfoFilter {
        self.filter.dial_info_filter
    }
}

impl NodeRefOperateTrait for FilteredNodeRef {
    fn operate<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&RoutingTableInner, &BucketEntryInner) -> T,
    {
        let routing_table = self.registry.routing_table();
        let inner = &*routing_table.inner.read();
        self.entry.with(inner, f)
    }

    fn operate_mut<T, F>(&self, f: F) -> T
    where
        F: FnOnce(&mut RoutingTableInner, &mut BucketEntryInner) -> T,
    {
        let routing_table = self.registry.routing_table();
        let inner = &mut *routing_table.inner.write();
        self.entry.with_mut(inner, f)
    }
}

impl NodeRefCommonTrait for FilteredNodeRef {}

impl Clone for FilteredNodeRef {
    fn clone(&self) -> Self {
        self.entry.ref_count.fetch_add(1u32, Ordering::AcqRel);

        Self {
            registry: self.registry.clone(),
            entry: self.entry.clone(),
            filter: self.filter,
            sequencing: self.sequencing,
            #[cfg(feature = "tracking")]
            track_id: self.entry.write().track(),
        }
    }
}

impl fmt::Display for FilteredNodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let node_id_str = if let Some(best_node_id) = self.entry.with_inner(|e| e.best_node_id()) {
            best_node_id.to_string()
        } else if let Some(node_id) = self.entry.with_inner(|e| e.node_ids().first().cloned()) {
            node_id.to_string()
        } else {
            "*ERROR*".to_string()
        };

        let mut out = node_id_str;

        let sstr = self.sequencing.to_string();
        if !sstr.is_empty() {
            out += "+";
            out += &sstr;
        }

        let fstr = self.filter.to_string();
        if !fstr.is_empty() {
            out += "/";
            out += &fstr;
        }

        write!(f, "{}", out)
    }
}

impl fmt::Debug for FilteredNodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilteredNodeRef")
            .field("node_ids", &self.entry.with_inner(|e| e.node_ids()))
            .field("filter", &self.filter)
            .field("sequencing", &self.sequencing)
            .finish()
    }
}

impl Drop for FilteredNodeRef {
    fn drop(&mut self) {
        #[cfg(feature = "tracking")]
        self.entry.write().untrack(self.track_id);

        // drop the noderef and queue a bucket kick if it was the last one
        let new_ref_count = self.entry.ref_count.fetch_sub(1u32, Ordering::AcqRel) - 1;
        if new_ref_count == 0 {
            // get node ids with inner unlocked because nothing could be referencing this entry now
            // and we don't know when it will get dropped, possibly inside a lock
            let node_ids = self.entry.with_inner(|e| e.node_ids());
            self.routing_table().queue_bucket_kicks(node_ids);
        }
    }
}
