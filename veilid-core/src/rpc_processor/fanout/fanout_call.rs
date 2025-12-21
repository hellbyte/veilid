use super::*;

impl_veilid_log_facility!("fanout");

#[derive(Debug)]
struct FanoutContext<'a> {
    fanout_queue: FanoutQueue<'a>,
    result: FanoutResult,
    done: FanoutDoneDisposition,
    stop_source: Option<StopSource>,
}

#[derive(Debug, Copy, Clone, Default)]
pub enum FanoutResultKind {
    #[default]
    Incomplete,
    Timeout,
    Consensus,
    Exhausted,
}
impl FanoutResultKind {
    pub fn is_incomplete(&self) -> bool {
        matches!(self, Self::Incomplete)
    }
}

#[derive(Clone, Debug, Default)]
pub struct FanoutResult {
    /// How the fanout completed
    pub kind: FanoutResultKind,
    /// The set of nodes that counted toward consensus
    /// (for example, had the most recent value for this subkey)
    pub consensus_nodes: Vec<NodeRef>,
    /// Which nodes accepted the request
    pub value_nodes: Vec<NodeRef>,
}

impl fmt::Display for FanoutResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kc = match self.kind {
            FanoutResultKind::Incomplete => "I",
            FanoutResultKind::Timeout => "T",
            FanoutResultKind::Consensus => "C",
            FanoutResultKind::Exhausted => "E",
        };
        if f.alternate() {
            write!(
                f,
                "{}:{}[{}]",
                kc,
                self.consensus_nodes.len(),
                self.consensus_nodes
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        } else {
            write!(f, "{}:{}", kc, self.consensus_nodes.len())
        }
    }
}

pub fn debug_fanout_results(results: &[FanoutResult]) -> String {
    let mut col = 0;
    let mut out = String::new();
    let mut left = results.len();
    for r in results {
        if col == 0 {
            out += "    ";
        }
        let sr = format!("{}", r);
        out += &sr;
        out += ",";
        col += 1;
        left -= 1;
        if col == 32 && left != 0 {
            col = 0;
            out += "\n"
        }
    }
    out
}

#[derive(Debug)]
pub struct FanoutCallOutput {
    pub peer_info_list: Vec<Arc<PeerInfo>>,
    pub disposition: FanoutCallDisposition,
}

#[derive(Debug, Clone, Copy)]
pub enum FanoutQueueMode {
    ThrottleAtConsensus,
    Unthrottled,
}

const THROTTLE_DURATION_PERCENT: u64 = 33;

/// The return type of the fanout call routine
#[derive(Debug, Copy, Clone)]
pub enum FanoutCallDisposition {
    /// The call routine timed out
    Timeout,
    /// The call routine returned an invalid result
    Invalid,
    /// The called node rejected the rpc request but may have returned more nodes
    Rejected,
    /// The called node accepted the rpc request and may have returned more nodes,
    /// but we don't count the result toward our consensus
    Stale,
    /// The called node accepted the rpc request and may have returned more nodes,
    /// counting the result toward our consensus
    Accepted,
    /// The called node accepted the rpc request and may have returned more nodes,
    /// returning a newer value that indicates we should restart our consensus
    AcceptedNewerRestart,
    /// The called node accepted the rpc request and may have returned more nodes,
    /// returning a newer value that indicates our current consensus is stale and should be ignored,
    /// and counting the result toward a new consensus
    AcceptedNewer,
}

/// The return type of the fanout done routine
#[derive(Debug, Copy, Clone)]
pub enum FanoutDoneDisposition {
    /// Finish immediately without completing operations
    DoneEarly,
    /// Finish when all operations are complete
    Done,
    /// Not done yet
    NotDone,
}

/// The return type of a fanout processor lane
enum FanoutProcessorReturn {
    DoneEarly,
    Done,
    Tick,
}

pub type FanoutCallResult = Result<FanoutCallOutput, RPCError>;
pub type FanoutPeerInfoFilter = Arc<dyn (Fn(Arc<PeerInfo>) -> bool) + Send + Sync>;
pub type FanoutCheckDone = Arc<dyn (Fn(&FanoutResult) -> FanoutDoneDisposition) + Send + Sync>;
pub type FanoutCallRoutine =
    Arc<dyn (Fn(NodeRef) -> PinBoxFutureStatic<FanoutCallResult>) + Send + Sync>;

pub fn empty_fanout_peer_info_filter() -> FanoutPeerInfoFilter {
    Arc::new(|_| true)
}

pub fn capability_fanout_peer_info_filter(caps: Vec<VeilidCapability>) -> FanoutPeerInfoFilter {
    Arc::new(move |pi| pi.node_info().has_all_capabilities(&caps))
}

/// Contains the logic for generically searching the Veilid routing table for a set of nodes and applying an
/// RPC operation that eventually converges on satisfactory result, or times out and returns some
/// unsatisfactory but acceptable result. Or something.
///
/// The algorithm starts by creating a 'closest_nodes' working set of the nodes closest to some node id currently in our routing table
/// If has pluggable callbacks:
///  * 'check_done' - for checking for a termination condition
///  * 'call_routine' - routine to call for each node that performs an operation and may add more nodes to our closest_nodes set
///
/// The algorithm is parameterized by:
///  * 'node_count' - the number of nodes to keep in the closest_nodes set
///  * 'fanout' - the number of concurrent calls being processed at the same time
///  * 'consensus_count' - the number of nodes in the processed queue that need to be in the 'Accepted' state before we terminate the fanout early.
///
/// The algorithm returns early if 'check_done' returns some value, or if an error is found during the process.
/// If the algorithm times out, a Timeout result is returned, however operations will still have been performed and a
/// timeout is not necessarily indicative of an algorithmic 'failure', just that no definitive stopping condition was found
/// in the given time
pub(crate) struct FanoutCall<'a> {
    name: String,
    routing_table: &'a RoutingTable,
    hash_coordinate: HashCoordinate,
    node_count: usize,
    fanout_tasks: usize,
    consensus_count: usize,
    timeout: TimestampDuration,
    peer_info_filter: FanoutPeerInfoFilter,
    call_routine: FanoutCallRoutine,
    check_done: FanoutCheckDone,
}

impl VeilidComponentRegistryAccessor for FanoutCall<'_> {
    fn registry(&self) -> VeilidComponentRegistry {
        self.routing_table.registry()
    }
}

impl<'a> FanoutCall<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        routing_table: &'a RoutingTable,
        hash_coordinate: HashCoordinate,
        node_count: usize,
        fanout_tasks: usize,
        consensus_count: usize,
        timeout: TimestampDuration,
        peer_info_filter: FanoutPeerInfoFilter,
        call_routine: FanoutCallRoutine,
        check_done: FanoutCheckDone,
    ) -> Self {
        Self {
            name,
            routing_table,
            hash_coordinate,
            node_count,
            fanout_tasks,
            consensus_count,
            timeout,
            peer_info_filter,
            call_routine,
            check_done,
        }
    }

    #[instrument(level = "trace", target = "fanout", skip_all)]
    fn evaluate_done(&self, ctx: &mut FanoutContext) -> FanoutDoneDisposition {
        // If we already finished, just return
        if !matches!(ctx.done, FanoutDoneDisposition::NotDone) {
            return ctx.done;
        }

        // Calculate fanout result so far
        let fanout_result = ctx.fanout_queue.with_nodes(|nodes, sorted_nodes| {
            // Count up nodes we have seen in order and see if our closest nodes have a consensus
            let mut consensus: Option<bool> = None;
            let mut consensus_nodes: Vec<NodeRef> = vec![];
            let mut value_nodes: Vec<NodeRef> = vec![];
            for sn in sorted_nodes {
                let node = nodes.get(sn).unwrap();
                match node.status.stage() {
                    FanoutNodeStage::Queued | FanoutNodeStage::InProgress => {
                        // Still have a closer node to do before reaching consensus,
                        // or are doing it still, then wait until those are done
                        if consensus.is_none() {
                            consensus = Some(false);
                        }
                    }
                    FanoutNodeStage::Timeout
                    | FanoutNodeStage::Rejected
                    | FanoutNodeStage::Disqualified => {
                        // Node does not count toward consensus or value node list
                    }
                    FanoutNodeStage::Stale => {
                        // Node does not count toward consensus but does count toward value node list
                        value_nodes.push(node.node_ref.clone());
                    }
                    FanoutNodeStage::Accepted => {
                        // Node counts toward consensus and value node list
                        value_nodes.push(node.node_ref.clone());

                        consensus_nodes.push(node.node_ref.clone());
                        if consensus.is_none() && consensus_nodes.len() >= self.consensus_count {
                            consensus = Some(true);
                        }
                    }
                }
            }

            // If we have reached sufficient consensus, return done
            match consensus {
                Some(true) => FanoutResult {
                    kind: FanoutResultKind::Consensus,
                    consensus_nodes,
                    value_nodes,
                },
                Some(false) => FanoutResult {
                    kind: FanoutResultKind::Incomplete,
                    consensus_nodes,
                    value_nodes,
                },
                None => FanoutResult {
                    kind: FanoutResultKind::Exhausted,
                    consensus_nodes,
                    value_nodes,
                },
            }
        });

        let done = (self.check_done)(&fanout_result);
        ctx.result = fanout_result;
        ctx.done = done;
        if !matches!(done, FanoutDoneDisposition::NotDone) {
            drop(ctx.stop_source.take())
        }
        done
    }

    #[instrument(level = "trace", target = "fanout", skip_all)]
    async fn fanout_processor(
        &self,
        lane_name: String,
        context: &Mutex<FanoutContext<'_>>,
    ) -> Result<FanoutProcessorReturn, RPCError> {
        // Make a stop token to break out when we're done
        let stop_token = context
            .lock()
            .stop_source
            .as_ref()
            .ok_or_else(|| RPCError::internal("should have stop source"))?
            .token();

        // Loop until we have a result or are done
        loop {
            // Put in a work request
            let work_receiver = {
                let mut context_locked = context.lock();
                veilid_log!(self debug "{}[{}]: Requesting work", self.name, lane_name);
                context_locked
                    .fanout_queue
                    .request_work(lane_name.clone())?
            };

            // Wait around for some work to do
            let Ok(Ok(next_node)) = work_receiver
                .recv_async()
                .timeout_at(stop_token.clone())
                .await
            else {
                // If we don't have a node to process, or we are being told to stop, stop fanning out
                veilid_log!(self debug "{}[{}]: Lane done", self.name, lane_name);
                break Ok(FanoutProcessorReturn::Done);
            };

            // Do the call for this node
            match (self.call_routine)(next_node.clone()).await {
                Ok(output) => {
                    // Filter returned nodes
                    let filtered_v: Vec<Arc<PeerInfo>> = output
                        .peer_info_list
                        .into_iter()
                        .filter(|pi| {
                            if !(self.peer_info_filter)(pi.clone()) {
                                return false;
                            }
                            true
                        })
                        .collect();

                    // Call succeeded
                    // Register the returned nodes and add them to the fanout queue in sorted order
                    let new_nodes = self
                        .routing_table
                        .register_nodes_with_peer_info_list(filtered_v);

                    // Update queue
                    {
                        let mut context_locked = context.lock();
                        let cur_ts = Timestamp::now_non_decreasing();

                        // Process disposition of the output of the fanout call routine
                        match output.disposition {
                            FanoutCallDisposition::Timeout => {
                                context_locked.fanout_queue.timeout(next_node, cur_ts);
                            }
                            FanoutCallDisposition::Rejected => {
                                context_locked.fanout_queue.rejected(next_node, cur_ts);
                            }
                            FanoutCallDisposition::Accepted => {
                                context_locked.fanout_queue.accepted(next_node, cur_ts);
                            }
                            FanoutCallDisposition::AcceptedNewerRestart => {
                                context_locked.fanout_queue.all_accepted_to_queued(cur_ts);
                                context_locked.fanout_queue.accepted(next_node, cur_ts);
                            }
                            FanoutCallDisposition::AcceptedNewer => {
                                context_locked.fanout_queue.all_accepted_to_stale(cur_ts);
                                context_locked.fanout_queue.accepted(next_node, cur_ts);
                            }
                            FanoutCallDisposition::Invalid => {
                                context_locked.fanout_queue.disqualified(next_node, cur_ts);
                            }
                            FanoutCallDisposition::Stale => {
                                context_locked.fanout_queue.stale(next_node, cur_ts);
                            }
                        }

                        // Add any new nodes
                        context_locked.fanout_queue.update(&new_nodes, cur_ts);

                        // See if we're done before going back for more processing
                        match self.evaluate_done(&mut context_locked) {
                            FanoutDoneDisposition::DoneEarly => {
                                veilid_log!(self debug "{}[{}]: Fanout done, terminating all other lanes", self.name, lane_name);
                                break Ok(FanoutProcessorReturn::DoneEarly);
                            }
                            FanoutDoneDisposition::Done => {
                                veilid_log!(self debug "{}[{}]: Fanout done, letting other lanes complete", self.name, lane_name);
                                break Ok(FanoutProcessorReturn::Done);
                            }
                            FanoutDoneDisposition::NotDone => {
                                veilid_log!(self debug "{}[{}]: Work done, continuing lane processing", self.name, lane_name);
                            }
                        }
                    }
                }
                Err(e) => {
                    veilid_log!(self debug "{}[{}]: Error occurred: {}", self.name, lane_name, e);
                    break Err(e);
                }
            };
        }
    }

    #[instrument(level = "trace", target = "fanout", skip_all)]
    fn init_closest_nodes(
        &self,
        context: &mut FanoutContext,
        cur_ts: Timestamp,
    ) -> Result<(), RPCError> {
        // Get the 'node_count' closest nodes to the key out of our routing table
        let closest_nodes = {
            let peer_info_filter = self.peer_info_filter.clone();
            let filter = Box::new(
                move |rti: &RoutingTableInner,
                      opt_entry: Option<Arc<BucketEntry>>,
                      _cur_ts: Timestamp| {
                    // Exclude our own node
                    if opt_entry.is_none() {
                        return false;
                    }
                    let entry = opt_entry.unwrap();

                    // Filter entries
                    entry.with(rti, |_rti, e| {
                        let Some(peer_info) = e.get_peer_info(RoutingDomain::PublicInternet) else {
                            return false;
                        };
                        // Ensure only things that are valid/signed in the PublicInternet domain are returned
                        if peer_info.signatures().is_empty() {
                            return false;
                        }

                        // Check our node info filter
                        if !(peer_info_filter)(peer_info.clone()) {
                            return false;
                        }

                        true
                    })
                },
            ) as RoutingTableEntryFilter;
            let filters = VecDeque::from([filter]);

            let transform = |_rti: &RoutingTableInner, v: Option<Arc<BucketEntry>>| {
                NodeRef::new(self.routing_table.registry(), v.unwrap().clone())
            };

            self.routing_table
                .find_preferred_closest_nodes(
                    self.node_count,
                    self.hash_coordinate.clone(),
                    filters,
                    transform,
                )
                .map_err(RPCError::invalid_format)?
        };

        context.fanout_queue.update(&closest_nodes, cur_ts);

        Ok(())
    }

    #[instrument(level = "trace", target = "fanout", skip_all)]
    pub async fn run(
        &self,
        init_fanout_queue: Vec<NodeRef>,
        fanout_queue_mode: FanoutQueueMode,
    ) -> Result<FanoutResult, RPCError> {
        // Create context for this run
        let node_sort = Box::new(make_closest_node_id_sort(self.hash_coordinate.clone()));
        let context = Arc::new(Mutex::new(FanoutContext {
            fanout_queue: FanoutQueue::new(
                self.name.clone(),
                self.routing_table.registry(),
                self.hash_coordinate.kind(),
                node_sort,
                self.consensus_count,
                match fanout_queue_mode {
                    FanoutQueueMode::ThrottleAtConsensus => Some(
                        self.timeout
                            .saturating_mul(THROTTLE_DURATION_PERCENT)
                            .div(100),
                    ),
                    FanoutQueueMode::Unthrottled => None,
                },
            ),
            result: FanoutResult {
                kind: FanoutResultKind::Incomplete,
                consensus_nodes: vec![],
                value_nodes: vec![],
            },
            done: FanoutDoneDisposition::NotDone,
            stop_source: Some(StopSource::new()),
        }));

        // Get timeout in milliseconds
        let timeout_ms = self.timeout.millis_u32().map_err(RPCError::internal)?;

        // Initialize closest nodes list
        {
            let context_locked = &mut *context.lock();
            let cur_ts = Timestamp::now_non_decreasing();

            self.init_closest_nodes(context_locked, cur_ts)?;

            // Ensure we include the most recent nodes
            context_locked
                .fanout_queue
                .update(&init_fanout_queue, cur_ts);

            // Do a quick check to see if we're already done
            if !matches!(
                self.evaluate_done(context_locked),
                FanoutDoneDisposition::NotDone
            ) {
                return Ok(core::mem::take(&mut context_locked.result));
            }
        }

        // Ticker to pump the queue
        let stop_token = context
            .lock()
            .stop_source
            .as_ref()
            .ok_or_else(|| RPCError::internal("should have stop source"))?
            .token();
        let make_tick_future = || {
            let stop_token = stop_token.clone();
            pin_dyn_future!(async move {
                if sleep(100).timeout_at(stop_token).await.is_err() {
                    return Ok(FanoutProcessorReturn::Done);
                }
                Ok(FanoutProcessorReturn::Tick)
            })
        };

        // If not, do the fanout
        let mut unord = FuturesUnordered::new();
        {
            // Spin up 'fanout' tasks to process the fanout
            for n in 0..self.fanout_tasks {
                unord.push(pin_dyn_future!(
                    self.fanout_processor(format!("lane#{}", n), &context)
                ));
            }
            // Add the initial timer tick task
            unord.push(make_tick_future());
        }

        // Wait for them to complete
        match timeout(
            timeout_ms,
            async {
                loop {
                    if let Some(res) = unord.next().in_current_span().await {
                        match res {
                            Ok(FanoutProcessorReturn::DoneEarly) => {
                                // Stop all lanes immediately
                                break Ok(());
                            }
                            Ok(FanoutProcessorReturn::Done) => {
                                // Lane finished but trying to finish other lanes
                            }
                            Ok(FanoutProcessorReturn::Tick) => {
                                // Timer tick to push more work to the processor lanes
                                let context_locked = &mut *context.lock();
                                let cur_ts = Timestamp::now_non_decreasing();
                                context_locked.fanout_queue.send_more_work(cur_ts);

                                // Set up next tick if there's still stuff processing
                                if !unord.is_empty() {
                                    unord.push(make_tick_future());
                                }
                            }
                            Err(e) => {
                                break Err(e);
                            }
                        }
                    } else {
                        break Ok(());
                    }
                }
            }
            .in_current_span(),
        )
        .await
        {
            Ok(Ok(())) => {
                // Finished, either by exhaustion or consensus,
                // time to return whatever value we came up with
                let context_locked = &mut *context.lock();

                // Print final queue
                veilid_log!(self debug "{}: Finished FanoutQueue:\n{}", self.name, context_locked.fanout_queue);

                return Ok(core::mem::take(&mut context_locked.result));
            }
            Ok(Err(e)) => {
                // Fanout died with an error
                return Err(e);
            }
            Err(_) => {
                // Timeout, do one last evaluate with remaining nodes in timeout state
                let context_locked = &mut *context.lock();
                let cur_ts = Timestamp::now_non_decreasing();
                context_locked
                    .fanout_queue
                    .all_unfinished_to_timeout(cur_ts);

                // Print final queue
                veilid_log!(self debug "{}: Timeout FanoutQueue:\n{}", self.name, context_locked.fanout_queue);

                // Final evaluate
                if !matches!(
                    self.evaluate_done(context_locked),
                    FanoutDoneDisposition::NotDone,
                ) {
                    // Last-chance value returned at timeout
                    return Ok(core::mem::take(&mut context_locked.result));
                }

                // We definitely weren't done, so this is just a plain timeout
                let mut result = core::mem::take(&mut context_locked.result);
                result.kind = FanoutResultKind::Timeout;
                return Ok(result);
            }
        }
    }
}
