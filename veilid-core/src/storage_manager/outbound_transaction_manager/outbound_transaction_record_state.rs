use super::*;

/// Parameters for creating an OutboundTransactionRecord
#[derive(Clone, Debug)]
pub(in crate::storage_manager) struct OutboundTransactionRecordParams {
    /// The record key being transacted over
    pub record_key: RecordKey,
    /// The signer key being used to authenticate the transaction
    pub signing_keypair: KeyPair,
    /// Consensus count required for this record to transact
    pub required_strict_consensus_count: usize,
    /// Safety selection to use for this record
    pub safety_selection: SafetySelection,
}

/// Stage consensus for record state across all node transactions
#[derive(Clone, Debug)]
pub(in crate::storage_manager) struct OutboundTransactionRecordStageConsensus {
    /// The best consensus stage we could come up with for this record
    pub stage: OutboundTransactionStage,
    /// The list of node transactions that should be rolled back at this point
    pub node_xids_to_rollback: Vec<NodeTransactionId>,
    /// The list of node transactions that should be dropped at this point
    pub node_xids_to_drop: Vec<NodeTransactionId>,
}

/// Which node transaction ids at what stage
type StageConsensusMap = HashMap<OutboundTransactionStage, Vec<NodeTransactionId>>;

/// Filter for get_transact_command_nodes
type GetTransactCommandNodesFilter<'a> = Box<dyn Fn(&'a NodeTransaction) -> bool + 'a>;

/// State per record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(in crate::storage_manager) struct OutboundTransactionRecordState {
    /// The record key being transacted over
    record_key: RecordKey,
    /// The signer key being used to authenticate the transaction
    signing_keypair: KeyPair,
    /// Transactions per node
    node_transactions: Vec<NodeTransaction>,
    /// Snapshot of maximum sequence numbers per subkey on the network at the time of transaction begin
    begin_network_seqs: Vec<ValueSeqNum>,
    /// The timestamp of when the transaction record was created
    created_ts: Timestamp,
    /// Consensus count required for this record to transact
    required_strict_consensus_count: usize,
    /// Safety selection to use for this record
    safety_selection: SafetySelection,
    /// Descriptor for the record
    /// Record may not exist locally until after the transaction, so this descriptor may have come from the network.
    descriptor: Option<Arc<SignedValueDescriptor>>,
    /// Schema for the record
    schema: Option<DHTSchema>,
    /// Snapshot of local record
    #[serde(skip)]
    local_snapshot: Option<Arc<RecordSnapshot>>,
    /// The last desired value for subkeys we have tried to set
    desired_subkeys: BTreeMap<ValueSubkey, Arc<SignedValueData>>,
    /// Consensus result of remote snapshot subkeys (newer subkeys returned, and gets)
    current_consensus: OutboundTransactionConsensus,
    /// Consensus result of remote subkey state upon transaction commit (sets)
    updated_consensus: OutboundTransactionConsensus,
}

impl fmt::Display for OutboundTransactionRecordState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} created@{} signer={} safety_selection={:?}\nnode_transactions:\n{}\nbegin_network_seqs:{}\n{}{}{}{}",
            self.record_key,
            self.created_ts,
            self.signing_keypair.key(),
            self.safety_selection,
            self.node_transactions
                .iter()
                .map(|x| format!("  {}", x))
                .collect::<Vec<_>>()
                .join("\n"),
            self.begin_network_seqs.to_table_string(),
            if let Some(local_snapshot) = self.local_snapshot.clone() {
                let local_seqs = local_snapshot.seqs();
                format!("local_seqs: {}\n", local_seqs.to_table_string())
            } else {
                "".to_string()
            },
            if !self.desired_subkeys.is_empty() {
                let desired_subkeys = self
                    .desired_subkeys
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "#{}={}",
                            k,
                            v.value_data().seq()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!("desired_subkeys: {}\n", desired_subkeys)
            } else {
                "".to_string()
            },
            if !self.current_consensus.is_empty() {
                format!("current_subkey_states: {}\n", &self.current_consensus)
            } else {
                "".to_string()
            },
            if !self.updated_consensus.is_empty() {
                format!("updated_subkey_states: {}\n", &self.updated_consensus)
            } else {
                "".to_string()
            }
        )
    }
}

impl OutboundTransactionRecordState {
    pub(super) fn new(params: OutboundTransactionRecordParams) -> Self {
        Self {
            record_key: params.record_key,
            signing_keypair: params.signing_keypair,
            safety_selection: params.safety_selection,
            node_transactions: vec![],
            begin_network_seqs: vec![],
            created_ts: Timestamp::now(),
            required_strict_consensus_count: params.required_strict_consensus_count,
            descriptor: None,
            schema: None,
            local_snapshot: None,
            desired_subkeys: Default::default(),
            current_consensus: OutboundTransactionConsensus::new(),
            updated_consensus: OutboundTransactionConsensus::new(),
        }
    }

    /// Calculate the consensus of the node transactions to determine what this record's effective stage is
    /// and actions to perform to reconcile the transaction for this record
    pub fn stage_consensus(&self) -> Option<OutboundTransactionRecordStageConsensus> {
        // If we have no node transactions, this is at an Init stage
        if self.node_transactions.is_empty() {
            return None;
        }

        // Count up what stages we are at with each node transaction
        let stage_consensus_map = self.get_stage_consensus_map();

        // Find a singular consensus
        let stage = {
            let mut opt_best_stage = None;
            for (st, stn) in stage_consensus_map.iter().map(|(st, stn)| (*st, stn)) {
                if stn.len() >= self.required_strict_consensus_count {
                    if opt_best_stage.is_none() {
                        opt_best_stage = Some(st);
                    } else {
                        // If more than one stage has met the strict consensus, then we say this is Failed
                        opt_best_stage = Some(OutboundTransactionStage::Failed);
                        break;
                    }
                }
            }
            // If no stage has met the strict consensus, this is also failed
            opt_best_stage.unwrap_or(OutboundTransactionStage::Failed)
        };

        // If the consensus stage has some requred state at this point, validate it
        let stage = match stage {
            OutboundTransactionStage::Failed
            | OutboundTransactionStage::Rollback
            | OutboundTransactionStage::Commit => {
                // Nothing to validate here, stage is the same
                stage
            }
            OutboundTransactionStage::Begin | OutboundTransactionStage::End => {
                if self.local_snapshot.is_none() {
                    // Snapshot lost when this deserialized, so stage is now Failed
                    OutboundTransactionStage::Failed
                } else if self.descriptor.is_none() {
                    // Descriptor was never found, stage is Failed
                    OutboundTransactionStage::Failed
                } else {
                    // We have what we need, stage is the same
                    stage
                }
            }
        };

        // Now that we know what our stage stage is, determine what should to be done
        // to move on to the next operation cleanly
        let stage_consensus = match stage {
            OutboundTransactionStage::Failed
            | OutboundTransactionStage::Rollback
            | OutboundTransactionStage::Commit => {
                // Failed stage means we can only Rollback node transactions so we get a Rollback consensus
                // Rollback stage means we have a consensus at Rollback but there may be other nodes that should get rolled back
                // Commit stage means we have a consensus at Commit but there may be other nodes that should get rolled back
                // At these stages there is no point to dropping node transactions because they will -all- get dropped at termination

                let node_xids_to_rollback = Self::get_all_rollbacks_internal(&stage_consensus_map);
                OutboundTransactionRecordStageConsensus {
                    stage,
                    node_xids_to_rollback,
                    node_xids_to_drop: vec![],
                }
            }
            OutboundTransactionStage::Begin => {
                // Begin stage means we have a consensus at Begin, but there may be other nodes that should get rolled back and dropped, or just dropped

                // Find all rollback-capable (not finished) node transaction ids and already-rolled-back ids so we can drop them
                let mut force_fail = false;
                let mut node_xids_to_rollback = vec![];
                let mut node_xids_to_drop = vec![];
                for (st, stn) in stage_consensus_map
                    .iter()
                    .map(|(st, stn)| (*st, stn.as_slice()))
                {
                    match st {
                        OutboundTransactionStage::Failed => {
                            // Roll back and drop any failed nodes
                            node_xids_to_rollback.extend_from_slice(stn);
                            node_xids_to_drop.extend_from_slice(stn);
                        }
                        OutboundTransactionStage::Rollback => {
                            // If nodes are already rolled back then just drop them
                            node_xids_to_drop.extend_from_slice(stn);
                        }
                        OutboundTransactionStage::Begin => {
                            // Keep the consensus nodes
                        }
                        OutboundTransactionStage::End | OutboundTransactionStage::Commit => {
                            // If some nodes ended or committed but we are still somehow at a consensus of Begin,
                            // roll back everything and move to a failed state
                            force_fail = true;
                        }
                    }
                }

                if force_fail {
                    // Don't bother dropping any nodes, only roll back everything
                    let node_xids_to_rollback =
                        Self::get_all_rollbacks_internal(&stage_consensus_map);
                    OutboundTransactionRecordStageConsensus {
                        stage: OutboundTransactionStage::Failed,
                        node_xids_to_rollback,
                        node_xids_to_drop: vec![],
                    }
                } else {
                    // Return the stage consensus
                    OutboundTransactionRecordStageConsensus {
                        stage,
                        node_xids_to_rollback,
                        node_xids_to_drop,
                    }
                }
            }
            OutboundTransactionStage::End => {
                // End stage means we have a consensus at End, but there may be other nodes that should get rolled back and dropped, or just dropped

                // Find all rollback-capable (not finished) node transaction ids and already-rolled-back ids so we can drop them
                let mut force_fail = false;
                let mut node_xids_to_rollback = vec![];
                let mut node_xids_to_drop = vec![];
                for (st, stn) in stage_consensus_map
                    .iter()
                    .map(|(st, stn)| (*st, stn.as_slice()))
                {
                    match st {
                        OutboundTransactionStage::Failed | OutboundTransactionStage::Begin => {
                            // Roll back and drop any failed nodes or nodes still at the begin stage
                            node_xids_to_rollback.extend_from_slice(stn);
                            node_xids_to_drop.extend_from_slice(stn);
                        }
                        OutboundTransactionStage::Rollback => {
                            // If nodes are already rolled back then just drop them
                            node_xids_to_drop.extend_from_slice(stn);
                        }
                        OutboundTransactionStage::End => {
                            // Keep the consensus nodes
                        }
                        OutboundTransactionStage::Commit => {
                            // If some nodes committed but we are still somehow at a consensus of End,
                            // roll back everything and move to a failed state
                            force_fail = true;
                        }
                    }
                }

                if force_fail {
                    // Don't bother dropping any nodes, only roll back everything
                    let node_xids_to_rollback =
                        Self::get_all_rollbacks_internal(&stage_consensus_map);
                    OutboundTransactionRecordStageConsensus {
                        stage: OutboundTransactionStage::Failed,
                        node_xids_to_rollback,
                        node_xids_to_drop: vec![],
                    }
                } else {
                    // Return the stage consensus
                    OutboundTransactionRecordStageConsensus {
                        stage,
                        node_xids_to_rollback,
                        node_xids_to_drop,
                    }
                }
            }
        };

        Some(stage_consensus)
    }

    /// Force-rollback everything that isn't done and return a stage consensus describing the actions
    pub(super) fn get_all_rollbacks(&self) -> Vec<NodeTransactionId> {
        let stage_consensus_map = self.get_stage_consensus_map();
        Self::get_all_rollbacks_internal(&stage_consensus_map)
    }

    pub(super) fn get_all_rollbacks_internal(
        stage_consensus_map: &StageConsensusMap,
    ) -> Vec<NodeTransactionId> {
        // Don't bother dropping any nodes, only roll back everything that can be rolled back
        let mut node_xids_to_rollback = vec![];
        for (st, stn) in stage_consensus_map
            .iter()
            .map(|(st, stn)| (*st, stn.as_slice()))
        {
            if !matches!(
                st,
                OutboundTransactionStage::Rollback | OutboundTransactionStage::Commit
            ) {
                node_xids_to_rollback.extend_from_slice(stn);
            }
        }

        // Return the stage consensus
        node_xids_to_rollback
    }

    pub(super) fn get_stage_consensus_map(&self) -> StageConsensusMap {
        // Count up what stages we are at with each node transaction
        let mut stage_consensus_map = StageConsensusMap::new();
        for nt in self.node_transactions.iter() {
            let node_transaction_stage = nt.stage();
            let node_xid = nt.node_xid().clone();

            let node_xids = stage_consensus_map
                .entry(node_transaction_stage)
                .or_default();
            node_xids.push(node_xid);
        }

        stage_consensus_map
    }

    #[expect(dead_code)]
    pub fn created_ts(&self) -> Timestamp {
        self.created_ts
    }

    pub fn opt_expiration(&self) -> Option<Timestamp> {
        self.node_transactions
            .iter()
            .map(|x| x.opt_expiration())
            .reduce(|a, b| match (a, b) {
                (None, None) => None,
                (None, Some(b)) => Some(b),
                (Some(a), None) => Some(a),
                (Some(a), Some(b)) => Some(a.min(b)),
            })
            .flatten()
    }

    pub fn stage_ts(&self) -> Timestamp {
        self.node_transactions
            .iter()
            .map(|x| x.stage_ts())
            .reduce(|a, b| a.max(b))
            .unwrap_or(self.created_ts)
    }

    pub fn record_key(&self) -> &RecordKey {
        &self.record_key
    }

    pub fn signing_keypair(&self) -> &KeyPair {
        &self.signing_keypair
    }

    pub fn safety_selection(&self) -> &SafetySelection {
        &self.safety_selection
    }

    pub fn required_strict_consensus_count(&self) -> usize {
        self.required_strict_consensus_count
    }

    pub fn prepare(&mut self, routing_table: &RoutingTable, cur_ts: Timestamp) {
        self.node_transactions
            .retain_mut(|x| x.prepare(routing_table, cur_ts));
    }

    fn sort_node_transactions(
        opaque_record_key: &OpaqueRecordKey,
        node_transactions: &mut [NodeTransaction],
    ) {
        node_transactions.sort_by(|a, b| {
            let dist_a = opaque_record_key
                .to_hash_coordinate()
                .distance(&a.node_id().to_hash_coordinate());
            let dist_b = opaque_record_key
                .to_hash_coordinate()
                .distance(&b.node_id().to_hash_coordinate());

            dist_a.cmp(&dist_b)
        });
    }

    pub fn update_descriptor(
        &mut self,
        descriptor: Arc<SignedValueDescriptor>,
    ) -> VeilidAPIResult<()> {
        let schema = descriptor.schema()?;
        if let Some(prev_descriptor) = self.descriptor.clone() {
            if prev_descriptor != descriptor {
                apibail_internal!(
                    "mismatched descriptor {:?} != {:?}",
                    prev_descriptor,
                    descriptor
                );
            }
        }
        self.descriptor = Some(descriptor);
        self.schema = Some(schema);
        Ok(())
    }

    pub fn descriptor(&self) -> Option<Arc<SignedValueDescriptor>> {
        self.descriptor.clone()
    }
    pub fn schema(&self) -> Option<&DHTSchema> {
        self.schema.as_ref()
    }

    pub fn update_begin_network_seqs(&mut self, seqs: Vec<ValueSeqNum>) -> VeilidAPIResult<()> {
        let Some(schema) = &self.schema else {
            apibail_internal!("should have schema before seqs");
        };

        if seqs.len() != schema.subkey_count() {
            apibail_internal!(
                "mismatched subkey count {} != {}",
                seqs.len(),
                schema.subkey_count()
            );
        }

        if self.begin_network_seqs.is_empty() {
            self.begin_network_seqs = seqs;
        } else {
            if seqs.len() != self.begin_network_seqs.len() {
                apibail_internal!(
                    "mismatched subkey count that should have been verified already {} != {}",
                    seqs.len(),
                    schema.subkey_count()
                );
            }
            for (ri_seq, seq) in self.begin_network_seqs.iter_mut().zip(seqs.into_iter()) {
                ri_seq.max_assign(seq)
            }
        }

        Ok(())
    }

    pub fn begin_network_seq(&self, subkey: ValueSubkey) -> VeilidAPIResult<ValueSeqNum> {
        self.begin_network_seqs
            .get(usize::try_from(subkey).map_err(VeilidAPIError::internal)?)
            .copied()
            .ok_or_else(|| VeilidAPIError::internal("subkey out of range"))
    }

    pub fn set_local_snapshot(&mut self, local_snapshot: Arc<RecordSnapshot>) {
        self.local_snapshot = Some(local_snapshot);
    }

    pub fn local_snapshot(&self) -> Option<Arc<RecordSnapshot>> {
        self.local_snapshot.clone()
    }

    pub fn new_node_transaction(
        &mut self,
        params: NodeTransactionParams,
    ) -> VeilidAPIResult<&mut NodeTransaction> {
        let node_xid = NodeTransactionId::new(
            params.node_ref.node_ids().get(params.kind).unwrap(),
            params.xid,
        );
        if self.get_node_transaction(&node_xid).is_some() {
            apibail_internal!("node transaction already exists");
        }

        self.node_transactions.push(NodeTransaction::new(
            node_xid.clone(),
            params.node_ref,
            params.expiration,
        ));
        Self::sort_node_transactions(&self.record_key.opaque(), &mut self.node_transactions);

        self.get_node_transaction_mut(&node_xid)
            .ok_or_else(|| VeilidAPIError::internal("can't get node transaction we just made"))
    }

    pub fn get_node_transaction<'a>(
        &'a self,
        node_xid: &NodeTransactionId,
    ) -> Option<&'a NodeTransaction> {
        self.node_transactions
            .iter()
            .find(|nx| nx.node_xid() == node_xid)
    }

    pub fn get_node_transaction_mut<'a>(
        &'a mut self,
        node_xid: &NodeTransactionId,
    ) -> Option<&'a mut NodeTransaction> {
        self.node_transactions
            .iter_mut()
            .find(|nx| nx.node_xid() == node_xid)
    }

    #[expect(dead_code)]
    pub fn get_node_transactions(&self) -> &[NodeTransaction] {
        &self.node_transactions
    }

    pub fn get_node_transactions_mut(&mut self) -> &mut [NodeTransaction] {
        &mut self.node_transactions
    }

    #[expect(dead_code)]
    pub fn get_node_xids<B: FromIterator<NodeTransactionId>>(&self) -> B {
        self.node_transactions
            .iter()
            .map(|nt| nt.node_xid().clone())
            .collect::<B>()
    }

    pub fn get_transact_command_nodes<'a>(
        &'a self,
        opt_node_xids: Option<&[NodeTransactionId]>,
        opt_filter: Option<GetTransactCommandNodesFilter<'a>>,
    ) -> VeilidAPIResult<OutboundTransactCommandNodes> {
        let out = self
            .node_transactions
            .iter()
            .filter(|nt| {
                if !opt_node_xids
                    .map(|node_xids| node_xids.contains(nt.node_xid()))
                    .unwrap_or(true)
                {
                    return false;
                }

                if let Some(filter) = opt_filter.as_ref() {
                    if !filter(nt) {
                        return false;
                    }
                }

                true
            })
            .map(|nt| (nt.node_xid().clone(), nt.node_ref()))
            .collect::<Vec<_>>();

        if let Some(node_xids) = opt_node_xids {
            if node_xids.len() != out.len() {
                apibail_internal!(
                    "tried to get command nodes for node xid not in record. requested: {:?} returned: {:?}",
                    node_xids, out
                );
            }
        }

        Ok(out)
    }

    pub fn set_desired_subkey(&mut self, subkey: ValueSubkey, value: Arc<SignedValueData>) {
        self.desired_subkeys.insert(subkey, value);
    }

    pub fn current_consensus(&self) -> &OutboundTransactionConsensus {
        &self.current_consensus
    }

    pub fn current_consensus_mut(&mut self) -> &mut OutboundTransactionConsensus {
        &mut self.current_consensus
    }

    pub fn updated_consensus(&self) -> &OutboundTransactionConsensus {
        &self.updated_consensus
    }

    pub fn updated_consensus_mut(&mut self) -> &mut OutboundTransactionConsensus {
        &mut self.updated_consensus
    }

    pub fn current_subkey_get_result(&self, subkey: ValueSubkey) -> VeilidAPIResult<GetResult> {
        let opt_descriptor = self.descriptor();
        let opt_snapshot_value = match &self.local_snapshot {
            Some(local_snapshot) => local_snapshot.subkey_value_data(subkey)?,
            None => None,
        };

        let opt_state_value = self
            .current_consensus
            .get(subkey)
            .and_then(|ss| ss.opt_value.clone());

        let opt_value = match (opt_snapshot_value, opt_state_value) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => {
                let a_seq = a.value_data().seq();
                let b_seq = b.value_data().seq();

                if a_seq > b_seq {
                    Some(a)
                } else if a_seq < b_seq {
                    Some(b)
                } else {
                    // Always defer to the network copy if conflicting or equal
                    Some(b)
                }
            }
        };

        Ok(GetResult {
            opt_value,
            opt_descriptor,
        })
    }

    pub fn local_commit_results(
        &self,
    ) -> VeilidAPIResult<Vec<(ValueSubkey, Arc<SignedValueData>)>> {
        let Some(max_subkey) = self.schema().map(|s| s.max_subkey()) else {
            return Ok(vec![]);
        };

        let mut out = vec![];
        for subkey in 0..=max_subkey {
            let opt_current_value = self
                .current_consensus
                .get(subkey)
                .and_then(|sc| sc.opt_value.clone());
            let opt_updated_value = self
                .updated_consensus
                .get(subkey)
                .and_then(|sc| sc.opt_value.clone());

            if let Some(updated_value) = opt_updated_value {
                out.push((subkey, updated_value));
            } else if let Some(current_value) = opt_current_value {
                let opt_snapshot_value = match &self.local_snapshot {
                    Some(local_snapshot) => local_snapshot.subkey_value_data(subkey)?,
                    None => None,
                };
                if let Some(snapshot_value) = opt_snapshot_value {
                    if current_value.value_data().seq() > snapshot_value.value_data().seq() {
                        out.push((subkey, current_value));
                    }
                } else {
                    out.push((subkey, current_value));
                }
            }
        }
        Ok(out)
    }

    pub fn remove_node_transactions(&mut self, node_xids: &[NodeTransactionId]) -> bool {
        let mut removed = false;
        for node_xid in node_xids {
            if let Some(remove_pos) = self
                .node_transactions
                .iter()
                .position(|x| x.node_xid() == node_xid)
            {
                self.node_transactions.remove(remove_pos);
                removed = true;
            }
        }
        removed
    }
}
