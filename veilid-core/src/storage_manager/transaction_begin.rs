use std::sync::Arc;

use super::*;

impl_veilid_log_facility!("stor");

/// The context of the outbound_transact_begin operation
struct OutboundTransactBeginContext {
    /// The descriptor we have
    pub opt_descriptor: Option<Arc<SignedValueDescriptor>>,
    /// The best sequence numbers so far
    pub seqs: Vec<ValueSeqNum>,
    /// The set of nodes that returned a transaction id
    pub node_transaction_params: Vec<NodeTransactionParams>,
}

/// parameters required to begin a transaction
pub(super) struct OutboundTransactBeginParams {
    /// The record key being transacted
    pub opaque_record_key: OpaqueRecordKey,
    /// The safety selection used for the transaction
    pub safety_selection: SafetySelection,
    /// The signer used to sign the transaction
    pub signing_keypair: KeyPair,
}

/// The result of the outbound_transact_begin operation
#[derive(Debug, Clone)]
pub(super) struct OutboundTransactBeginResult {
    /// The record key being transacted
    pub opaque_record_key: OpaqueRecordKey,
    /// Fanout result
    pub fanout_result: FanoutResult,
    /// The set of nodes that returned a transaction id
    pub node_transaction_params: Vec<NodeTransactionParams>,
    /// The combined list of newest sequence numbers from the transaction nodes
    pub seqs: Vec<ValueSeqNum>,
    /// The descriptor for the record
    pub descriptor: Arc<SignedValueDescriptor>,
}

/// The result of the inbound_transact_begin operation
#[derive(Clone, Debug)]
pub(crate) enum InboundTransactBeginResult {
    /// Value transacted successfully
    Success(TransactBeginSuccess),
    /// Transaction unavailable due to limits
    TransactionUnavailable,
    /// Descriptor required but not provided,
    NeedDescriptor,
}

/// The result of a single successful transaction begin
#[derive(Debug, Clone)]
pub(crate) struct TransactBeginSuccess {
    /// Transaction id
    pub transaction_id: InboundTransactionId,
    /// Expiration timestamp
    pub expiration: Timestamp,
    /// Descriptor
    pub opt_descriptor: Option<Arc<SignedValueDescriptor>>,
    /// Sequence numbers for record
    pub seqs: Vec<ValueSeqNum>,
}

impl StorageManager {
    ////////////////////////////////////////////////////////////////////////

    /// Perform a transact begin query on the network for a single record
    /// This routine uses fanout and stores the fanout result and individual transaction ids in xxxx
    #[instrument(level = "trace", target = "dht", skip_all, err)]
    pub(super) async fn outbound_transact_begin(
        &self,
        params: OutboundTransactBeginParams,
    ) -> VeilidAPIResult<OutboundTransactBeginResult> {
        let OutboundTransactBeginParams {
            opaque_record_key,
            safety_selection,
            signing_keypair,
        } = params;

        let routing_domain = RoutingDomain::PublicInternet;

        // Get the DHT parameters for 'TransactBegin'
        let config = self.config();
        let (key_count, consensus_count, fanout, timeout) = (
            config.network.dht.max_find_node_count as usize,
            config.network.dht.set_value_count as usize,
            config.network.dht.set_value_fanout as usize,
            TimestampDuration::new_ms(config.network.rpc.timeout_ms.into()),
        );

        // Get the nodes we know are caching this value to seed the fanout
        let init_fanout_queue = self
            .get_value_nodes(&opaque_record_key)?
            .unwrap_or_default()
            .into_iter()
            .filter(|x| {
                x.node_info(routing_domain)
                    .map(|ni| ni.has_all_capabilities(&[VEILID_CAPABILITY_DHT]))
                    .unwrap_or_default()
            })
            .collect();

        // Get the descriptor for this record if we have it
        let opt_descriptor = {
            let local_record_store = self.get_local_record_store()?;
            local_record_store.with_record(&opaque_record_key, |record| record.descriptor())?
        };

        // Make operation context
        let context = Arc::new(Mutex::new(OutboundTransactBeginContext {
            opt_descriptor,
            seqs: vec![],
            node_transaction_params: vec![],
        }));

        let descriptor_cache = self.descriptor_cache.clone();

        // Routine to call to generate fanout
        let call_routine = {
            let context = context.clone();
            let registry = self.registry();
            let opaque_record_key = opaque_record_key.clone();
            let safety_selection = safety_selection.clone();
            let descriptor_cache = descriptor_cache.clone();
            let signing_keypair = signing_keypair.clone();
            Arc::new(
                move |next_node: NodeRef| -> PinBoxFutureStatic<FanoutCallResult> {
                    let context = context.clone();
                    let registry = registry.clone();
                    let opaque_record_key = opaque_record_key.clone();
                    let safety_selection = safety_selection.clone();
                    let descriptor_cache = descriptor_cache.clone();
                    let signing_keypair = signing_keypair.clone();
                    Box::pin(async move {
                        let rpc_processor = registry.rpc_processor();

                        // check the cache to see if we should send the descriptor
                        let node_id = next_node.node_ids().get(opaque_record_key.kind()).unwrap();
                        let dc_key = DescriptorCacheKey{ opaque_record_key: opaque_record_key.clone(), node_id };
                        let mut descriptor_mode = DescriptorMode::new(descriptor_cache.lock().get(&dc_key).is_none(), context.lock().opt_descriptor.clone());

                        // send across the wire, with a retry if the remote needed the descriptor
                        let tva = loop {
                            // send across the wire
                            let tva = match
                            rpc_processor
                                .rpc_call_transact_begin(
                                    Destination::direct(next_node.routing_domain_filtered(routing_domain)).with_safety(safety_selection.clone()),
                                    opaque_record_key.clone(),
                                    descriptor_mode.clone(),
                                    signing_keypair.clone(),
                                )
                                .await? {
                                NetworkResult::Timeout => {
                                    return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Timeout});
                                }
                                NetworkResult::ServiceUnavailable(_) |
                                NetworkResult::NoConnection(_)  |
                                NetworkResult::AlreadyExists(_) |
                                NetworkResult::InvalidMessage(_) => {
                                    return Ok(FanoutCallOutput{peer_info_list: vec![], disposition: FanoutCallDisposition::Invalid});
                                }
                                NetworkResult::Value(v) => v
                            };

                            // Do a retry if we needed to send the descriptor
                            // (if the cache was wrong)
                            if tva.answer.accepted && tva.answer.descriptor_mode.is_want() {
                                match descriptor_mode {
                                    DescriptorMode::Want => {
                                        // If both sides want the descriptor but do not have it then the record does not exist
                                    }
                                    DescriptorMode::Have(signed_value_descriptor) => {
                                        // If the server wants the descriptor and we have it, then send it
                                        descriptor_mode = DescriptorMode::Send(signed_value_descriptor);

                                        veilid_log!(registry debug target:"network_result", "Retrying to send descriptor");
                                        continue;
                                    }
                                    DescriptorMode::Send(_) => {
                                        // If the server wants the descriptor and we already sent it, then something is wrong
                                        veilid_log!(registry error target:"network_result", "Got 'need_descriptor' when descriptor was already sent: node={} record_key={}", next_node, opaque_record_key);
                                    }
                                }
                            }

                            break tva;
                        };

                        let answer = tva.answer;

                        // Check if we got an accepted result
                        if !answer.accepted {
                            // Return peers if we have some
                            veilid_log!(registry debug target:"network_result", "TransactBegin missed, fanout call returned peers {}", answer.peers.len());
                            return Ok(FanoutCallOutput{peer_info_list:answer.peers, disposition: FanoutCallDisposition::Rejected});
                        }

                        // If the node was close enough to accept the value
                        let mut ctx = context.lock();

                        // Get the descriptor and cache if we sent the descriptor or if we received one
                        let Some(descriptor) = ctx.opt_descriptor.clone().or(answer.descriptor_mode.opt_arc_descriptor()) else {
                            // Record does not exist
                            veilid_log!(registry debug target:"network_result", "TransactBegin record did not exist, fanout call returned peers {}", answer.peers.len());
                            return Ok(FanoutCallOutput{peer_info_list:answer.peers, disposition: FanoutCallDisposition::Rejected});
                        };
                        if descriptor_mode.is_send() || answer.descriptor_mode.is_send() || answer.descriptor_mode.is_have() {
                            descriptor_cache.lock().insert(dc_key,());
                        }

                        // Get the transaction id
                        let Some(xid) = answer.transaction_id else {
                            veilid_log!(registry debug target:"network_result", "TransactBegin accepted but returned no transaction id, try again later");
                            return Err(RPCError::try_again("transaction unavailable"));
                        };

                        let schema = match descriptor.schema() {
                            Ok(s) => s,
                            Err(_) => {
                                veilid_log!(registry debug target:"network_result", "TransactBegin received invalid schema");
                                return Ok(FanoutCallOutput{peer_info_list:vec![], disposition: FanoutCallDisposition::Invalid});
                            }
                        };
                        let subkey_count = schema.subkey_count();

                        // Get the sequence number state at the point of the transaction
                        if answer.seqs.len() != subkey_count {
                            veilid_log!(registry debug target:"network_result", "wrong number of seqs returned {} (wanted {})",
                                answer.seqs.len(),
                                subkey_count);
                            return Ok(FanoutCallOutput{peer_info_list: answer.peers, disposition: FanoutCallDisposition::Invalid});
                        }

                        veilid_log!(registry debug target:"network_result", "Got transaction id and seqs back: xid={}, len={}", xid, answer.seqs.len());

                        // Update descriptor in context so we don't send/want it more than necessary
                        ctx.opt_descriptor = Some(descriptor);

                        // Add transaction id node to list
                        ctx.node_transaction_params.push(NodeTransactionParams{ kind: opaque_record_key.kind(), xid, node_ref: next_node.clone(), expiration: answer.expiration});

                        // If we have a prior seqs list, merge in the new seqs
                        if ctx.seqs.is_empty() {
                            ctx.seqs = answer.seqs.clone()
                        } else {
                            for pair in ctx.seqs.iter_mut().zip(answer.seqs.iter()) {
                                let ctx_seq = pair.0;
                                let answer_seq = *pair.1;

                                ctx_seq.max_assign(answer_seq);
                            }
                        }

                        // Return peers if we have some
                        veilid_log!(registry debug target:"network_result", "TransactBegin fanout call returned peers {}", answer.peers.len());

                        // Transact doesn't actually use the fanout queue consensus tracker
                        Ok(FanoutCallOutput { peer_info_list: answer.peers, disposition: FanoutCallDisposition::Accepted})
                    }.instrument(tracing::trace_span!("outbound_begin_transact_value fanout call"))) as PinBoxFuture<FanoutCallResult>
                },
            )
        };

        // Routine to call to check if we're done at each step
        let check_done = {
            Arc::new(move |fanout_result: &FanoutResult| {
                match fanout_result.kind {
                    FanoutResultKind::Incomplete => {
                        // Keep going
                        FanoutDoneDisposition::NotDone
                    }
                    FanoutResultKind::Timeout | FanoutResultKind::Exhausted => {
                        // Signal we're done
                        FanoutDoneDisposition::DoneEarly
                    }
                    FanoutResultKind::Consensus => {
                        // Signal we're done
                        FanoutDoneDisposition::Done
                    }
                }
            })
        };

        // Call the fanout
        let routing_table = self.routing_table();
        let fanout_call = FanoutCall::new(
            format!("outbound_transact_begin({})", Timestamp::now_increasing()),
            &routing_table,
            opaque_record_key.to_hash_coordinate(),
            key_count,
            fanout,
            consensus_count,
            timeout,
            capability_fanout_peer_info_filter(vec![VEILID_CAPABILITY_DHT]),
            call_routine,
            check_done,
        );

        let fanout_result = fanout_call
            .run(init_fanout_queue, FanoutQueueMode::ThrottleAtConsensus)
            .await?;

        let ctx = context.lock();

        veilid_log!(self debug target: "network_result", "TransactBegin Fanout: {:#}", fanout_result);

        let descriptor = ctx.opt_descriptor.clone().unwrap();
        let seqs = if fanout_result.value_nodes.is_empty() {
            vec![ValueSeqNum::NONE; descriptor.schema().unwrap().subkey_count()]
        } else {
            ctx.seqs.clone()
        };

        let result = OutboundTransactBeginResult {
            opaque_record_key,
            fanout_result,
            node_transaction_params: ctx.node_transaction_params.clone(),
            seqs,
            descriptor,
        };

        Ok(result)
    }

    ////////////////////////////////////////////////////////////////////////

    /// Handle a received 'TransactBegin' query
    #[instrument(level = "debug", target = "dht", ret(Display), err, fields(duration, __VEILID_LOG_KEY = self.log_key(), opt_descriptor = opt_descriptor.is_some()), skip(self, opt_descriptor))]
    pub async fn inbound_transact_begin(
        &self,
        opaque_record_key: OpaqueRecordKey,
        opt_descriptor: Option<SignedValueDescriptor>,
        want_descriptor: bool,
        signing_member_id: MemberId,
    ) -> VeilidAPIResult<NetworkResult<InboundTransactBeginResult>> {
        record_duration_fut(async {
            // Can't provide descriptor and want descriptor
            if opt_descriptor.is_some() && want_descriptor {
                return VeilidAPIResult::Ok(NetworkResult::invalid_message(
                    "can't provide descriptor and want descriptor",
                ));
            }

            let remote_record_store = self.get_remote_record_store()?;

            remote_record_store
                .begin_inbound_transaction(
                    &opaque_record_key,
                    opt_descriptor,
                    want_descriptor,
                    signing_member_id,
                )
                .await
                .map(NetworkResult::value)
        })
        .await
    }
}
