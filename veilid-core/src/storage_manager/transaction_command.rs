use futures_util::StreamExt as _;

use super::*;

impl_veilid_log_facility!("stor");

pub(super) type OutboundTransactCommandNodes = Vec<(NodeTransactionId, NodeRef)>;

/// parameters required to perform a command on a transaction
#[derive(Debug, Clone)]
pub(super) struct OutboundTransactCommandParams {
    /// The record key being transacted
    pub opaque_record_key: OpaqueRecordKey,
    /// The safety selection used with the transaction
    pub safety_selection: SafetySelection,
    /// Nodes and transaction ids to use
    pub nodes: OutboundTransactCommandNodes,
    /// The command to execute on each node
    pub command: TransactCommand,
    /// Parameter for the command (sequence numbers)
    pub opt_seqs: Option<Vec<ValueSeqNum>>,
    /// Parameter for the command (subkey number)
    pub opt_subkey: Option<ValueSubkey>,
    /// Parameter for the command (value)
    pub opt_value: Option<Arc<SignedValueData>>,
}

/// The result of the outbound_transact_command operation
#[derive(Debug, Clone)]
pub(super) struct OutboundTransactCommandPerNodeResult {
    /// The node transaction id this is for
    pub node_transaction_id: NodeTransactionId,
    /// True if the transaction is still valid
    pub transaction_valid: bool,
    /// Return from the command (sequence numbers)
    #[expect(dead_code)]
    pub opt_seqs: Option<Vec<ValueSeqNum>>,
    /// Return from the command (subkey number)
    pub opt_subkey: Option<ValueSubkey>,
    /// Return from the command (value)
    pub opt_value: Option<Arc<SignedValueData>>,
    /// Updated expiration to apply
    pub opt_expiration: Option<Timestamp>,
}

/// The result of the outbound_transact_command operation
#[derive(Debug, Clone)]
pub(super) struct OutboundTransactCommandResult {
    /// Copy of the params used to produce these results
    pub params: OutboundTransactCommandParams,
    /// The results per node, in closest-to-the-record-key sorted order
    pub per_node_results: Vec<OutboundTransactCommandPerNodeResult>,
}

impl OutboundTransactCommandResult {
    pub fn get_command_node_xids(&self) -> HashSet<NodeTransactionId> {
        self.params
            .nodes
            .iter()
            .map(|x| x.0.clone())
            .collect::<HashSet<_>>()
    }
}

/// The result of the inbound_transact_command operation
#[derive(Clone, Debug)]
pub(crate) enum InboundTransactCommandResult {
    /// Value transacted successfully
    Success(TransactCommandSuccess),
    /// Transaction not valid
    InvalidTransaction,
    /// Invalid arguments
    InvalidArguments,
}

/// The result of a single successful transaction command
#[derive(Default, Debug, Clone)]
pub(crate) struct TransactCommandSuccess {
    /// Expiration timestamp
    pub expiration: Timestamp,
    /// Sequence numbers
    pub opt_seqs: Option<Vec<ValueSeqNum>>,
    /// Subkey
    pub opt_subkey: Option<ValueSubkey>,
    /// Value
    pub opt_value: Option<Arc<SignedValueData>>,
}

impl StorageManager {
    ////////////////////////////////////////////////////////////////////////

    /// Perform transact command queries on the network for a single record
    #[instrument(level = "trace", target = "dht", skip_all, err)]
    pub(super) async fn outbound_transact_command(
        &self,
        params: OutboundTransactCommandParams,
    ) -> VeilidAPIResult<OutboundTransactCommandResult> {
        let OutboundTransactCommandParams {
            opaque_record_key,
            safety_selection,
            nodes,
            command,
            opt_seqs,
            opt_subkey,
            opt_value,
        } = params.clone();

        let routing_domain = RoutingDomain::PublicInternet;

        // Pull the descriptor for this record
        let descriptor = {
            let local_record_store = self.get_local_record_store()?;
            local_record_store
                .with_record(&opaque_record_key, |record| record.descriptor())?
                .ok_or_else(|| VeilidAPIError::internal("record does not exist in transaction"))?
        };

        // Send all commands in parallel
        let mut unord = FuturesUnordered::new();
        for (node_transaction_id, node_ref) in nodes {
            let registry = self.registry();

            let descriptor = descriptor.clone();
            let opaque_record_key = opaque_record_key.clone();
            let safety_selection = safety_selection.clone();
            let opt_seqs = opt_seqs.clone();
            let opt_value = opt_value.clone();

            let fut = async move {
                let rpc_processor = registry.rpc_processor();

                let tva = network_result_value_or_log!(self Box::pin(rpc_processor
                    .rpc_call_transact_command(
                        Destination::direct(node_ref.routing_domain_filtered(routing_domain))
                            .with_safety(safety_selection.clone()),
                        opaque_record_key.clone(),
                        descriptor.clone(),
                        node_transaction_id.xid(),
                        command,
                        opt_seqs,
                        opt_subkey,
                        opt_value,
                    ).measure_debug(TimestampDuration::new_secs(5), veilid_log_dbg!(
                        self,
                        "StorageManager::outbound_transact_command rpc_call_transact_command"
                    )))
                    .await
                    .map_err(VeilidAPIError::from)? => [
                        format!(": {} key={} xid={:?}{}{}",
                            command,
                            opaque_record_key,
                            node_transaction_id,
                            if let Some(subkey) = opt_subkey {
                                format!(" #{}", subkey)
                            } else {
                                "".to_string()
                            },
                            if let Some(value) = &opt_value {
                                format!(" {}", value)
                            } else {
                                "".to_string()
                            }
                    ) ]
                {
                    return VeilidAPIResult::Ok(None);
                });

                if !tva.answer.transaction_valid {
                    veilid_log!(self debug target:"network_result", "Transaction was no longer valid: node={} record_key={}", node_ref, opaque_record_key);
                }

                let pnr = OutboundTransactCommandPerNodeResult {
                    node_transaction_id,
                    transaction_valid: tva.answer.transaction_valid,
                    opt_seqs: tva.answer.opt_seqs,
                    opt_subkey: tva.answer.opt_subkey,
                    opt_value: tva.answer.opt_value,
                    opt_expiration: tva.answer.opt_expiration,
                };
                Ok(Some(pnr))
            };

            unord.push(fut);
        }

        let mut per_node_results = vec![];
        while let Some(res) = unord.next().await {
            let res = res.inspect_err(|e| {
                veilid_log!(self error target:"network_result", "Error performing transaction command: {}", e);
            })?;

            if let Some(pnr) = res {
                per_node_results.push(pnr);
            }
        }

        // Sort per node results by distance from record to assist with strict consensus checking
        per_node_results.sort_by(|a, b| {
            let dist_a = opaque_record_key
                .to_hash_coordinate()
                .distance(&a.node_transaction_id.node_id().to_hash_coordinate());
            let dist_b = opaque_record_key
                .to_hash_coordinate()
                .distance(&b.node_transaction_id.node_id().to_hash_coordinate());

            dist_a.cmp(&dist_b)
        });

        Ok(OutboundTransactCommandResult {
            params,
            per_node_results,
        })
    }

    ////////////////////////////////////////////////////////////////////////

    /// Handle a received 'TransactCommand' query
    #[instrument(level = "debug", target = "dht", ret(Display), err, fields(duration, __VEILID_LOG_KEY = self.log_key(), opt_value.len = opt_value.as_ref().map(|x| x.value_data().data_size())), skip(self, opt_value, _opt_seqs))]
    pub async fn inbound_transact_command(
        &self,
        opaque_record_key: &OpaqueRecordKey,
        transaction_id: u64,
        command: TransactCommand,
        _opt_seqs: Option<Vec<ValueSeqNum>>,
        opt_subkey: Option<ValueSubkey>,
        opt_value: Option<Arc<SignedValueData>>,
    ) -> VeilidAPIResult<NetworkResult<InboundTransactCommandResult>> {
        record_duration_fut(async {
            let remote_record_store = self.get_remote_record_store()?;

            let transaction_id =
                match remote_record_store.lookup_inbound_transaction_id(transaction_id)? {
                    Some(id) => id,
                    None => {
                        return Ok(NetworkResult::value(
                            InboundTransactCommandResult::InvalidTransaction,
                        ));
                    }
                };

            let res = match command {
                TransactCommand::End => {
                    remote_record_store
                        .end_inbound_transaction(opaque_record_key, transaction_id)
                        .await?
                }
                TransactCommand::Commit => {
                    remote_record_store
                        .commit_inbound_transaction(opaque_record_key, transaction_id, || {
                            RemoteRecordDetail {}
                        })
                        .await?
                }
                TransactCommand::Rollback => {
                    remote_record_store
                        .rollback_inbound_transaction(opaque_record_key, transaction_id)
                        .await?
                }
                TransactCommand::Get => {
                    remote_record_store
                        .inbound_transaction_get(opaque_record_key, transaction_id, opt_subkey)
                        .await?
                }
                TransactCommand::Set => {
                    let Some(subkey) = opt_subkey else {
                        return Ok(NetworkResult::invalid_message("missing subkey"));
                    };
                    let Some(value) = opt_value else {
                        return Ok(NetworkResult::invalid_message("missing value"));
                    };
                    remote_record_store
                        .inbound_transaction_set(opaque_record_key, transaction_id, subkey, value)
                        .await?
                }
            };

            Ok(NetworkResult::value(res))
        })
        .await
    }
}
