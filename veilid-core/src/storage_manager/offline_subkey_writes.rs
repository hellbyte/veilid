use super::*;

impl_veilid_log_facility!("stor");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfflineSubkeyWrite {
    /// Safety selection to use when writing this record to the network
    pub safety_selection: SafetySelection,
    /// The subkeys that are queued up needing to be sent to the network in the background
    pub subkeys: ValueSubkeyRangeSet,
    /// The subkeys currently being sent to the network in the background
    #[serde(default)]
    pub subkeys_in_flight: ValueSubkeyRangeSet,
    /// The value data to send to the network if it is newer than what is in the local record store
    #[serde(default)]
    pub subkey_value_data: HashMap<ValueSubkey, Arc<SignedValueData>>,
}

impl StorageManager {
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub(super) fn add_offline_subkey_write_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: TypedRecordKey,
        subkey: ValueSubkey,
        safety_selection: SafetySelection,
        signed_value_data: Arc<SignedValueData>,
    ) {
        inner
            .offline_subkey_writes
            .entry(record_key)
            .and_modify(|x| {
                x.subkeys.insert(subkey);
                x.subkey_value_data
                    .insert(subkey, signed_value_data.clone());
            })
            .or_insert(OfflineSubkeyWrite {
                safety_selection,
                subkeys: ValueSubkeyRangeSet::single(subkey),
                subkeys_in_flight: ValueSubkeyRangeSet::new(),
                subkey_value_data: {
                    let mut subkey_value_data = HashMap::new();
                    subkey_value_data.insert(subkey, signed_value_data);
                    subkey_value_data
                },
            });
    }

    pub(super) fn get_offline_subkey_writes_subkey(
        &self,
        inner: &mut StorageManagerInner,
        record_key: TypedRecordKey,
        subkey: ValueSubkey,
        want_descriptor: bool,
    ) -> VeilidAPIResult<Option<GetResult>> {
        let Some(local_record_store) = inner.local_record_store.as_mut() else {
            apibail_not_initialized!();
        };
        let Some(osw) = inner.offline_subkey_writes.get(&record_key) else {
            return Ok(None);
        };
        let Some(signed_value_data) = osw.subkey_value_data.get(&subkey).cloned() else {
            return Ok(None);
        };
        let opt_descriptor = if want_descriptor {
            if let Some(descriptor) =
                local_record_store.with_record(record_key, |record| record.descriptor().clone())
            {
                Some(descriptor)
            } else {
                // Record not available
                return Ok(None);
            }
        } else {
            None
        };
        Ok(Some(GetResult {
            opt_value: Some(signed_value_data),
            opt_descriptor,
        }))
    }

    /// If an offline subkey write happens and then we find newer data on the network while
    /// waiting to process the offline subkey write, we should continue with it but use the
    /// newer data in place of the originally requested data. If the sequence number of the
    /// network data is the same, we defer to what is already on the network.
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub(super) fn remove_old_offline_subkey_writes_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: TypedRecordKey,
        subkey: ValueSubkey,
        signed_value_data: Arc<SignedValueData>,
    ) {
        // Get the offline subkey write record
        match inner.offline_subkey_writes.entry(record_key) {
            hashlink::linked_hash_map::Entry::Occupied(mut o) => {
                let finished = {
                    let osw = o.get_mut();
                    match osw.subkey_value_data.entry(subkey) {
                        std::collections::hash_map::Entry::Occupied(o) => {
                            // If new data has greater or equal sequence number to the
                            // offline set value, drop the old data from the offline subkey write
                            let old_data = o.get().value_data();
                            let new_data = signed_value_data.value_data();
                            if old_data != new_data && new_data.seq() >= old_data.seq() {
                                o.remove();
                                // Also, remove the subkey from queued offline subkey writes
                                // but leave it in-flight if it is in flight. That will get
                                // handled by finish_offline_subkey_writes_inner
                                osw.subkeys.remove(subkey);

                                veilid_log!(self debug "offline write overwritten by newer or different data from network: record_key={} subkey={} seq={}", record_key, subkey, signed_value_data.value_data().seq());
                            }
                        }
                        std::collections::hash_map::Entry::Vacant(_) => {}
                    }

                    // If we have no new work to do, and not still doing work, then this record is done
                    let finished = osw.subkeys.is_empty() && osw.subkeys_in_flight.is_empty();
                    if !finished {
                        // Remove any subkey value data that is no longer needed
                        let osw = o.get_mut();
                        osw.subkey_value_data.retain(|k, _| {
                            osw.subkeys.contains(*k) || osw.subkeys_in_flight.contains(*k)
                        });
                    }
                    finished
                };
                if finished {
                    veilid_log!(self debug "Offline write finished key {}", record_key);
                    o.remove();
                }
            }
            hashlink::linked_hash_map::Entry::Vacant(_) => {}
        }
    }

    /// When we finish a offline subkey write, we mark subkeys as no longer in-flight
    /// and if we didn't finish all the subkeys they are returned to the list of offline subkeys
    /// so we can try again later. If the data associated with the write is no longer necessary
    /// we can drop it.
    #[instrument(level = "trace", target = "stor", skip_all)]
    pub(super) fn finish_offline_subkey_writes_inner(
        &self,
        inner: &mut StorageManagerInner,
        record_key: TypedRecordKey,
        subkeys_written: ValueSubkeyRangeSet,
        subkeys_still_offline: ValueSubkeyRangeSet,
    ) {
        assert!(
            subkeys_written.is_disjoint(&subkeys_still_offline),
            "subkeys can not be written and still offline"
        );

        // Get the offline subkey write record
        match inner.offline_subkey_writes.entry(record_key) {
            hashlink::linked_hash_map::Entry::Occupied(mut o) => {
                let finished = {
                    let osw = o.get_mut();

                    // Now any left over are still offline, so merge them with any subkeys that have been added while we were working
                    osw.subkeys = osw.subkeys.union(&subkeys_still_offline);

                    // Remove subkeys that were successfully written from in_flight status
                    osw.subkeys_in_flight = osw.subkeys_in_flight.difference(&subkeys_written);

                    // If we have no new work to do, and not still doing work, then this record is done
                    let finished = osw.subkeys.is_empty() && osw.subkeys_in_flight.is_empty();
                    if !finished {
                        // Remove any subkey value data that is no longer needed
                        let osw = o.get_mut();
                        osw.subkey_value_data.retain(|k, _| {
                            osw.subkeys.contains(*k) || osw.subkeys_in_flight.contains(*k)
                        });
                    }
                    finished
                };
                if finished {
                    veilid_log!(self debug "offline subkey write finished key {}", record_key);
                    o.remove();
                }
            }
            hashlink::linked_hash_map::Entry::Vacant(_) => {
                veilid_log!(self warn "can't finish missing offline subkey write: ignoring key {}", record_key);
            }
        }
    }
}
