use super::{inspect_record::OutboundInspectValueResult, *};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RehydrateReport {
    /// The record key rehydrated
    opaque_record_key: OpaqueRecordKey,
    /// The requested range of subkeys to rehydrate if necessary
    subkeys: ValueSubkeyRangeSet,
    /// The requested consensus count,
    consensus_count: usize,
    /// The range of subkeys that actually could be rehydrated
    rehydrated: ValueSubkeyRangeSet,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct RehydrationRequest {
    pub subkeys: ValueSubkeyRangeSet,
    pub consensus_count: usize,
}

impl StorageManager {
    /// Add a background rehydration request
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub fn add_rehydration_request(
        &self,
        opaque_record_key: OpaqueRecordKey,
        subkeys: ValueSubkeyRangeSet,
        consensus_count: usize,
    ) {
        let req = RehydrationRequest {
            subkeys,
            consensus_count,
        };
        veilid_log!(self debug "Adding rehydration request: {} {:?}", opaque_record_key, req);
        let mut inner = self.inner.lock();
        inner
            .rehydration_requests
            .entry(opaque_record_key)
            .and_modify(|r| {
                r.subkeys = r.subkeys.union(&req.subkeys);
                r.consensus_count.max_assign(req.consensus_count);
            })
            .or_insert(req);
    }

    /// Sends the local copies of all of a record's subkeys back to the network
    /// Triggers a subkey update if the consensus on the subkey is less than
    /// the specified 'consensus_count'.
    /// The subkey updates are performed in the background if rehydration was
    /// determined to be necessary.
    /// If a newer copy of a subkey's data is available online, the background
    /// write will pick up the newest subkey data as it does the SetValue fanout
    /// and will drive the newest values to consensus.
    #[instrument(level = "trace", target = "stor", skip(self), ret)]
    pub(super) async fn rehydrate_record(
        &self,
        opaque_record_key: OpaqueRecordKey,
        subkeys: ValueSubkeyRangeSet,
        consensus_count: usize,
    ) -> VeilidAPIResult<RehydrateReport> {
        let Ok(_guard) = self.startup_lock.enter() else {
            apibail_not_initialized!();
        };

        let local_record_store = self.get_local_record_store()?;

        veilid_log!(self debug "Checking for record rehydration: {} {} @ consensus {}", opaque_record_key, subkeys, consensus_count);
        // Get subkey range for consideration
        let subkeys = if subkeys.is_empty() {
            ValueSubkeyRangeSet::full()
        } else {
            subkeys
        };

        let peek_lock = self
            .record_lock_table
            .peek_lock(opaque_record_key.clone())
            .await;

        let safety_selection = {
            let inner = self.inner.lock();
            // If this record key is in any transaction, disallow this operation at this time
            if inner
                .outbound_transaction_manager
                .get_transaction_by_record(&opaque_record_key)
                .is_some()
            {
                apibail_try_again!("not rehydrating while records is in transaction");
            }
            if let Some(opened_record) = inner.opened_records.get(&opaque_record_key) {
                opened_record.safety_selection()
            } else {
                // See if it's in the local record store
                let Some(safety_selection) = local_record_store
                    .with_record(&opaque_record_key, |rec| {
                        rec.detail().safety_selection.clone()
                    })?
                else {
                    apibail_key_not_found!(opaque_record_key);
                };
                safety_selection
            }
        };

        // See if the requested record is our local record store
        let local_inspect_result = self
            .handle_inspect_local_values_with_peek_lock(&peek_lock, subkeys.clone(), true)
            .await?;

        // Get rpc processor and drop mutex so we don't block while getting the value from the network
        if !self.dht_is_online() {
            apibail_try_again!("offline, try again later");
        };

        // Trim inspected subkey range to subkeys we have data for locally
        let local_inspect_result = local_inspect_result.strip_none_seqs();

        // Get the inspect record report from the network with only the subkeys for which we have
        // sequence numbers we have locally
        let outbound_inspect_result = self
            .outbound_inspect_value(
                &opaque_record_key,
                local_inspect_result.subkeys().clone(),
                safety_selection.clone(),
                InspectResult::default(),
                true,
            )
            .await?;

        // If online result had no subkeys, then trigger writing the entire record in the background
        if outbound_inspect_result.inspect_result.subkeys().is_empty()
            || outbound_inspect_result
                .inspect_result
                .opt_descriptor()
                .is_none()
        {
            return self
                .rehydrate_all_subkeys(
                    opaque_record_key,
                    subkeys,
                    consensus_count,
                    safety_selection,
                    local_inspect_result,
                )
                .await;
        }

        return self
            .rehydrate_required_subkeys(
                opaque_record_key,
                subkeys,
                consensus_count,
                safety_selection,
                local_inspect_result,
                outbound_inspect_result,
            )
            .await;
    }

    #[instrument(level = "trace", target = "stor", skip(self), ret, err)]
    pub(super) async fn rehydrate_all_subkeys(
        &self,
        opaque_record_key: OpaqueRecordKey,
        subkeys: ValueSubkeyRangeSet,
        consensus_count: usize,
        safety_selection: SafetySelection,
        local_inspect_result: InspectResult,
    ) -> VeilidAPIResult<RehydrateReport> {
        veilid_log!(self debug "Rehydrating all subkeys: record={} subkeys={}", opaque_record_key, subkeys);

        let mut rehydrated = ValueSubkeyRangeSet::new();
        for (n, subkey) in local_inspect_result.subkeys().iter().enumerate() {
            if local_inspect_result.seqs()[n].is_some() {
                self.add_offline_subkey_write(
                    opaque_record_key.clone(),
                    subkey,
                    safety_selection.clone(),
                );
                rehydrated.insert(subkey);
            }
        }

        if rehydrated.is_empty() {
            veilid_log!(self debug "Record wanted full rehydrating, but no subkey data available: record={} subkeys={}", opaque_record_key, subkeys);
        } else {
            veilid_log!(self debug "Record full rehydrating: record={} subkeys={} rehydrated={}", opaque_record_key, subkeys, rehydrated);
        }

        return Ok(RehydrateReport {
            opaque_record_key,
            subkeys,
            consensus_count,
            rehydrated,
        });
    }

    #[instrument(level = "trace", target = "stor", skip(self), ret, err)]
    pub(super) async fn rehydrate_required_subkeys(
        &self,
        opaque_record_key: OpaqueRecordKey,
        subkeys: ValueSubkeyRangeSet,
        consensus_count: usize,
        safety_selection: SafetySelection,
        local_inspect_result: InspectResult,
        outbound_inspect_result: OutboundInspectValueResult,
    ) -> VeilidAPIResult<RehydrateReport> {
        // For each subkey, determine if we should rehydrate it
        let mut rehydrated = ValueSubkeyRangeSet::new();
        for (n, subkey) in local_inspect_result.subkeys().iter().enumerate() {
            let local_seq = local_inspect_result.seqs()[n];
            if local_seq.is_none() {
                apibail_internal!("None sequence number found in local inspect results. Should have been stripped by strip_none_seqs(): {:?}", local_inspect_result);
            };

            // Find matching network sequence number position
            // (they must line up because subkey range is the same for both local and network inspect results)
            let mut rehydrate = false;

            let network_seq = outbound_inspect_result.inspect_result.seqs()[n];
            if local_seq > network_seq {
                // If our copy is newer, push it to the network
                rehydrate = true;
            } else {
                // If our copy is older or equal, rehydrate only if there isn't enough consensus
                let sfr = outbound_inspect_result
                    .subkey_fanout_results
                    .get(n)
                    .unwrap();

                // Does the online subkey have enough consensus?
                // If not, schedule it to be written in the background
                if sfr.consensus_nodes.len() < consensus_count {
                    rehydrate = true;
                }
            }

            if rehydrate {
                self.add_offline_subkey_write(
                    opaque_record_key.clone(),
                    subkey,
                    safety_selection.clone(),
                );
                rehydrated.insert(subkey);
            }
        }

        if rehydrated.is_empty() {
            veilid_log!(self debug "Record did not need rehydrating: record={} local_subkeys={}", opaque_record_key, local_inspect_result.subkeys());
        } else {
            veilid_log!(self debug "Record rehydrating: record={} local_subkeys={} rehydrated={}", opaque_record_key, local_inspect_result.subkeys(), rehydrated);
        }

        // Keep the list of nodes that returned a value for later reference
        let results_iter = outbound_inspect_result
            .inspect_result
            .subkeys()
            .iter()
            .map(ValueSubkeyRangeSet::single)
            .zip(outbound_inspect_result.subkey_fanout_results.into_iter());

        let existed = self.process_fanout_results(
            opaque_record_key.clone(),
            results_iter,
            false,
            self.config().network.dht.consensus_width as usize,
        )?;

        if !existed {
            veilid_log!(self debug
                "record was rehydrated but was deleted locally: {}",
                opaque_record_key
            );
        }

        Ok(RehydrateReport {
            opaque_record_key,
            subkeys,
            consensus_count,
            rehydrated,
        })
    }
}
