/// Fanout Queue
/// Keep a deque of unique nodes
/// Internally the 'front' of the list is the next out, and new nodes are added to the 'back' of the list
/// When passing in a 'cleanup' function, if it sorts the queue, the 'first' items in the queue are the 'next' out.
use super::*;

impl_veilid_log_facility!("fanout");
impl_veilid_component_accessors!(FanoutQueue<'_>);

/// The stage of a particular node we fanned out to
#[derive(Debug, Copy, Clone)]
pub enum FanoutNodeStage {
    /// Node that needs processing
    Queued,
    /// Node currently being processed
    InProgress,
    /// Node that timed out during processing
    Timeout,
    /// Node that rejected the query
    Rejected,
    /// Node that accepted the query with a current result
    Accepted,
    /// Node that accepted the query but had an older result
    Stale,
    /// Node that has been disqualified for being too far away from the key or are acting badly
    Disqualified,
}

/// The state of a particular node we fanned out to including the stage
/// and a linked list of transitions and their timestamps when they transitioned
#[derive(Debug, Clone)]
pub struct FanoutNodeStatus {
    stage: FanoutNodeStage,
    prev_status: Option<Box<FanoutNodeStatus>>,
    transition_ts: Timestamp,
    touch_ts: Timestamp,
}

impl FanoutNodeStatus {
    pub fn stage(&self) -> FanoutNodeStage {
        self.stage
    }

    pub fn queued(timestamp: Timestamp) -> Self {
        FanoutNodeStatus {
            stage: FanoutNodeStage::Queued,
            prev_status: None,
            transition_ts: timestamp,
            touch_ts: timestamp,
        }
    }

    pub fn transition(&mut self, stage: FanoutNodeStage, timestamp: Timestamp) {
        self.touch_ts = timestamp;

        let prev_status = Box::new(self.clone());
        self.stage = stage;
        self.prev_status = Some(prev_status);
        self.transition_ts = timestamp;
        self.touch_ts = timestamp;
    }

    pub fn touch(&mut self, timestamp: Timestamp) {
        match self.stage {
            FanoutNodeStage::Queued | FanoutNodeStage::InProgress => {
                self.touch_ts = timestamp;
            }
            FanoutNodeStage::Timeout
            | FanoutNodeStage::Rejected
            | FanoutNodeStage::Accepted
            | FanoutNodeStage::Stale
            | FanoutNodeStage::Disqualified => {
                // Don't touch these because they are not considered 'active'
            }
        }
    }
}

impl fmt::Display for FanoutNodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}{}{}",
            self.stage,
            if self.transition_ts == self.touch_ts {
                "".to_string()
            } else {
                format!("({})", self.touch_ts.duration_since(self.transition_ts))
            },
            if let Some(prev_status) = &self.prev_status {
                format!("<--{}", prev_status)
            } else {
                format!("@{}", self.transition_ts)
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct FanoutNode {
    pub node_ref: NodeRef,
    pub status: FanoutNodeStatus,
}

impl fmt::Display for FanoutNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut start_status = &self.status;
        while let Some(prev_status) = &start_status.prev_status {
            start_status = prev_status;
        }
        let total_duration = self
            .status
            .touch_ts
            .duration_since(start_status.transition_ts);
        write!(
            f,
            "{}{}: {}",
            self.node_ref,
            if total_duration.is_zero() {
                "".to_string()
            } else {
                format!(" ({})", total_duration)
            },
            self.status
        )
    }
}

pub type FanoutQueueSort<'a> = Box<dyn Fn(&NodeId, &NodeId) -> core::cmp::Ordering + Send + 'a>;

#[derive(Debug)]
struct FanoutWorkRequest {
    request_ts: Timestamp,
    lane_name: String,
    work_sender: FanoutWorkSender,
}

impl FanoutWorkRequest {
    fn new(lane_name: String, work_sender: FanoutWorkSender) -> Self {
        Self {
            request_ts: Timestamp::now_non_decreasing(),
            lane_name,
            work_sender,
        }
    }

    pub fn request_ts(&self) -> Timestamp {
        self.request_ts
    }

    pub fn lane_name(&self) -> String {
        self.lane_name.clone()
    }

    pub fn into_work_sender(self) -> FanoutWorkSender {
        self.work_sender
    }
}

pub type FanoutWorkReceiver = flume::Receiver<NodeRef>;
pub type FanoutWorkSender = flume::Sender<NodeRef>;

pub struct FanoutQueue<'a> {
    /// Name for debugging
    name: String,
    /// Link back to veilid component registry for logging
    registry: VeilidComponentRegistry,
    /// Crypto kind in use for this queue
    crypto_kind: CryptoKind,
    /// The status of all the nodes we have added so far
    nodes: HashMap<NodeId, FanoutNode>,
    /// Closer nodes to the record key are at the front of the list
    sorted_nodes: Vec<NodeId>,
    /// The sort function to use for the nodes
    node_sort: FanoutQueueSort<'a>,
    /// The channel to receive work requests to process
    work_request_sender: flume::Sender<FanoutWorkRequest>,
    work_request_receiver: flume::Receiver<FanoutWorkRequest>,
    /// Consensus count to use
    consensus_count: usize,
    /// Whether or not to stop handing out work when queue consensus is met
    /// Duration at which to start a new node when throttled
    opt_throttle_duration: Option<TimestampDuration>,
}

impl fmt::Debug for FanoutQueue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FanoutQueue")
            .field("crypto_kind", &self.crypto_kind)
            .field("nodes", &self.nodes)
            .field("sorted_nodes", &self.sorted_nodes)
            // .field("node_sort", &self.node_sort)
            .field("work_request_sender", &self.work_request_sender)
            .field("work_request_receiver", &self.work_request_receiver)
            .field("consensus_count", &self.consensus_count)
            .field("opt_throttle_duration", &self.opt_throttle_duration)
            .finish()
    }
}

impl fmt::Display for FanoutQueue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "nodes:\n{}",
            self.sorted_nodes
                .iter()
                .map(|x| format!("    {}: {}", x, self.nodes.get(x).unwrap_or_log().status))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl<'a> FanoutQueue<'a> {
    /// Create a queue for fanout candidates that have a crypto-kind compatible node id
    pub fn new(
        name: String,
        registry: VeilidComponentRegistry,
        crypto_kind: CryptoKind,
        node_sort: FanoutQueueSort<'a>,
        consensus_count: usize,
        opt_throttle_duration: Option<TimestampDuration>,
    ) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            name,
            registry,
            crypto_kind,
            nodes: HashMap::new(),
            sorted_nodes: Vec::new(),
            node_sort,
            work_request_sender: sender,
            work_request_receiver: receiver,
            consensus_count,
            opt_throttle_duration,
        }
    }

    /// Ask for more work when some is ready
    /// When work is ready it will be sent to work_sender so it can be received
    /// by the worker
    pub fn request_work(&mut self, lane_name: String) -> Result<FanoutWorkReceiver, RPCError> {
        let (work_sender, work_receiver) = flume::bounded(1);

        let work_request = FanoutWorkRequest::new(lane_name, work_sender);
        let request_ts = work_request.request_ts();

        self.work_request_sender
            .send(work_request)
            .map_err(RPCError::internal)?;

        // Send whatever work is available immediately
        self.send_more_work(request_ts);

        Ok(work_receiver)
    }

    /// Update the queue with changes, adding new nodes to a filtered and sorted
    /// list of fanout candidates and disqualifying nodes no longer needed
    pub fn update(&mut self, new_nodes: &[NodeRef], cur_ts: Timestamp) {
        for node_ref in new_nodes {
            // Ensure the node has a comparable key with our current crypto kind
            let Some(key) = node_ref.node_ids().get(self.crypto_kind) else {
                continue;
            };
            // Check if we have already seen this node before (only one call per node ever)
            if self.nodes.contains_key(&key) {
                continue;
            }
            // Add the new node
            self.nodes.insert(
                key.clone(),
                FanoutNode {
                    node_ref: node_ref.clone(),
                    status: FanoutNodeStatus::queued(cur_ts),
                },
            );
            self.sorted_nodes.push(key);
        }

        // Sort the node list
        self.sorted_nodes.sort_by(&self.node_sort);

        // Disqualify any nodes that can be
        self.disqualify(cur_ts);

        // Touch all node status
        for node in &self.sorted_nodes {
            self.nodes
                .get_mut(node)
                .unwrap_or_log()
                .status
                .touch(cur_ts);
        }

        veilid_log!(self debug
            "{}: FanoutQueue::update:\n{}{}\n",
            self.name,
            if new_nodes.is_empty() {
                "".to_string()
            } else {
                format!("new_nodes:{}\n",
                    new_nodes.iter().map(|x| format!("\n    {}", x))
                        .collect::<Vec<String>>()
                        .join(","))
            },
            self.to_string()
        );
    }

    /// Send next fanout candidates if available to whatever workers are ready
    pub fn send_more_work(&mut self, cur_ts: Timestamp) {
        // Get the next work and send it along
        let registry = self.registry();
        let mut working_toward_consensus = 0usize;
        let mut counting_consensus = true;

        let mut slow_nodes = Vec::<NodeRef>::new();

        for x in &mut self.sorted_nodes {
            // If there are no work receivers left then we should stop trying to send
            if self.work_request_receiver.is_empty() {
                break;
            }

            // If we have enough workers to get consensus don't send more work beyond that
            // unless we are unthrottled
            let mut throttle_unlock = false;
            if self.opt_throttle_duration.is_some() {
                if working_toward_consensus >= (self.consensus_count + slow_nodes.len()) {
                    break;
                } else if !slow_nodes.is_empty() {
                    throttle_unlock = true;
                }
            }

            // Get the queue entry and handle it appropriately
            let node = self.nodes.get_mut(x).unwrap_or_log();
            match node.status.stage {
                FanoutNodeStage::Queued => {
                    // Consensus counting stops at a queued node
                    counting_consensus = false;

                    // Send node to a work request
                    while let Ok(work_request) = self.work_request_receiver.try_recv() {
                        if throttle_unlock {
                            veilid_log!(registry debug "{}: Throttle unlock due to {} slow nodes", self.name, slow_nodes.len());
                        }
                        let lane_name = work_request.lane_name();
                        let request_ts = work_request.request_ts();
                        let work_sender = work_request.into_work_sender();

                        let node_ref = node.node_ref.clone();
                        if work_sender.send(node_ref).is_ok() {
                            // Queued -> InProgress
                            node.status.transition(FanoutNodeStage::InProgress, cur_ts);
                            veilid_log!(registry debug "{}: Queue sent more work {} after request to {} => {}", self.name, cur_ts.duration_since(request_ts), lane_name, node.node_ref);
                            break;
                        }
                    }
                }
                FanoutNodeStage::InProgress => {
                    // If something has been in progress for longer than 1/3 of the total timeout
                    // then we should send more work to start looking at another node
                    if let Some(throttle_duration) = self.opt_throttle_duration {
                        let stage_duration = cur_ts.duration_since(node.status.transition_ts);
                        if stage_duration > throttle_duration {
                            slow_nodes.push(node.node_ref.clone());
                        } else {
                            // If we would like this node to finish before we allow consensus
                            // then we have to stop count here until it has reached the throttle duration
                            counting_consensus = false;
                        }
                    }
                    if counting_consensus {
                        working_toward_consensus += 1;
                    }
                }
                FanoutNodeStage::Accepted | FanoutNodeStage::Stale => {
                    // Always consider these nodes as working toward consensus because they're done
                    if counting_consensus {
                        working_toward_consensus += 1;
                    }

                    // If this node was a slow node, remove it from the slow node list since we finished it
                    slow_nodes.retain(|x| !x.same_entry(&node.node_ref));
                }
                FanoutNodeStage::Timeout
                | FanoutNodeStage::Rejected
                | FanoutNodeStage::Disqualified => {
                    // Does not count toward consensus or stop counting it

                    // If this node was a slow node, remove it from the slow node list since we finished it
                    slow_nodes.retain(|x| !x.same_entry(&node.node_ref));
                }
            }
        }
    }

    /// Transition node InProgress -> Timeout
    pub fn timeout(&mut self, node_ref: NodeRef, cur_ts: Timestamp) {
        let key = node_ref.node_ids().get(self.crypto_kind).unwrap_or_log();
        let node = self.nodes.get_mut(&key).unwrap_or_log();
        if !matches!(node.status.stage, FanoutNodeStage::InProgress) {
            unreachable!("should be in progress");
        }
        node.status.transition(FanoutNodeStage::Timeout, cur_ts);
    }

    /// Transition node InProgress -> Rejected
    pub fn rejected(&mut self, node_ref: NodeRef, cur_ts: Timestamp) {
        let key = node_ref.node_ids().get(self.crypto_kind).unwrap_or_log();
        let node = self.nodes.get_mut(&key).unwrap_or_log();
        if !matches!(node.status.stage, FanoutNodeStage::InProgress) {
            unreachable!("should be in progress");
        }
        node.status.transition(FanoutNodeStage::Rejected, cur_ts);
        self.disqualify(cur_ts);
    }

    /// Transition node InProgress -> Accepted
    pub fn accepted(&mut self, node_ref: NodeRef, cur_ts: Timestamp) {
        let key = node_ref.node_ids().get(self.crypto_kind).unwrap_or_log();
        let node = self.nodes.get_mut(&key).unwrap_or_log();
        if !matches!(node.status.stage, FanoutNodeStage::InProgress) {
            unreachable!("should be in progress");
        }
        node.status.transition(FanoutNodeStage::Accepted, cur_ts);
    }

    /// Transition node InProgress -> Stale
    pub fn stale(&mut self, node_ref: NodeRef, cur_ts: Timestamp) {
        let key = node_ref.node_ids().get(self.crypto_kind).unwrap_or_log();
        let node = self.nodes.get_mut(&key).unwrap_or_log();
        if !matches!(node.status.stage, FanoutNodeStage::InProgress) {
            unreachable!("should be in progress");
        }
        node.status.transition(FanoutNodeStage::Stale, cur_ts);
    }

    /// Transition node InProgress -> Disqualified
    pub fn disqualified(&mut self, node_ref: NodeRef, cur_ts: Timestamp) {
        let key = node_ref.node_ids().get(self.crypto_kind).unwrap_or_log();
        let node = self.nodes.get_mut(&key).unwrap_or_log();
        if !matches!(node.status.stage, FanoutNodeStage::InProgress) {
            unreachable!("should be in progress");
        }
        node.status
            .transition(FanoutNodeStage::Disqualified, cur_ts);
    }

    /// Transition all Accepted -> Queued, in the event a newer value for consensus is found and we want to try again
    pub fn all_accepted_to_queued(&mut self, cur_ts: Timestamp) {
        for node in &mut self.nodes {
            if matches!(node.1.status.stage, FanoutNodeStage::Accepted) {
                node.1.status.transition(FanoutNodeStage::Queued, cur_ts);
            }
        }
    }

    /// Transition all Accepted -> Stale, in the event a newer value for consensus is found but we don't want to try again
    pub fn all_accepted_to_stale(&mut self, cur_ts: Timestamp) {
        for node in &mut self.nodes {
            if matches!(node.1.status.stage, FanoutNodeStage::Accepted) {
                node.1.status.transition(FanoutNodeStage::Stale, cur_ts);
            }
        }
    }

    /// Transition all Queued | InProgress -> Timeout, in the event that the fanout is being cut short by a timeout
    pub fn all_unfinished_to_timeout(&mut self, cur_ts: Timestamp) {
        for node in &mut self.nodes {
            if matches!(
                node.1.status.stage,
                FanoutNodeStage::Queued | FanoutNodeStage::InProgress
            ) {
                node.1.status.transition(FanoutNodeStage::Timeout, cur_ts);
            }
        }
    }

    /// Transition Queued -> Disqualified that are too far away from the record key
    fn disqualify(&mut self, cur_ts: Timestamp) {
        let mut consecutive_rejections = 0usize;
        let mut rejected_consensus = false;
        for node_id in &self.sorted_nodes {
            let node = self.nodes.get_mut(node_id).unwrap_or_log();
            if !rejected_consensus {
                if matches!(node.status.stage, FanoutNodeStage::Rejected) {
                    consecutive_rejections += 1;
                    if consecutive_rejections >= self.consensus_count {
                        rejected_consensus = true;
                    }
                    continue;
                } else {
                    consecutive_rejections = 0;
                }
            } else if matches!(node.status.stage, FanoutNodeStage::Queued) {
                node.status
                    .transition(FanoutNodeStage::Disqualified, cur_ts);
            }
        }
    }

    /// Review the nodes in the queue
    pub fn with_nodes<R, F: FnOnce(&HashMap<NodeId, FanoutNode>, &[NodeId]) -> R>(
        &self,
        func: F,
    ) -> R {
        func(&self.nodes, &self.sorted_nodes)
    }
}
