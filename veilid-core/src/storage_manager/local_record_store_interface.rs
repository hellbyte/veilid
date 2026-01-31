use super::*;

impl StorageManager {
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn handle_get_single_local_value(
        &self,
        opaque_record_key: &OpaqueRecordKey,
        subkey: ValueSubkey,
        want_descriptor: bool,
    ) -> VeilidAPIResult<GetResult> {
        let local_record_store = self.get_local_record_store()?;

        // See if it's in the local record store
        if let Some(get_result) = local_record_store
            .get_subkey(opaque_record_key, subkey, want_descriptor)
            .await?
        {
            return Ok(get_result);
        }

        Ok(GetResult::default())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn handle_offline_set_single_local_value_with_subkey_lock(
        &self,
        subkey_lock: &StorageManagerSubkeyLockGuard,
        value: Arc<SignedValueData>,
        safety_selection: SafetySelection,
        allow_offline: AllowOffline,
    ) -> VeilidAPIResult<()> {
        // Don't do this if we are disallowing offline writes
        if allow_offline == AllowOffline(false) {
            apibail_try_again!("offline, try again later");
        }

        let opaque_record_key = subkey_lock.record();
        let subkey = subkey_lock.subkey();

        veilid_log!(self debug "Writing subkey offline: {}:{} len={}", opaque_record_key, subkey, value.value_data().data().len() );

        // Write subkey to local store
        let local_record_store = self.get_local_record_store()?;
        local_record_store
            .set_single_subkey(
                &opaque_record_key,
                subkey,
                value.clone(),
                InboundWatchUpdateMode::NoUpdate,
                CommitActionFlushMode::Immediate,
            )
            .await?;

        // Ensure we come back to put this to the network later
        // (it may already be added but this ensures we try again)
        self.add_offline_subkey_write(opaque_record_key, subkey, safety_selection);

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn handle_set_single_local_value_with_subkey_lock(
        &self,
        subkey_lock: &StorageManagerSubkeyLockGuard,
        value: Arc<SignedValueData>,
    ) -> VeilidAPIResult<()> {
        let opaque_record_key = subkey_lock.record();
        let subkey = subkey_lock.subkey();

        // Remove any offline writes to this subkey since we're rewriting it
        {
            let mut inner = self.inner.lock();
            self.remove_offline_subkey_write_inner(&mut inner, &opaque_record_key, subkey);
        }

        // Write subkey to local store
        let local_record_store = self.get_local_record_store()?;
        local_record_store
            .set_single_subkey(
                &opaque_record_key,
                subkey,
                value.clone(),
                InboundWatchUpdateMode::NoUpdate,
                CommitActionFlushMode::Immediate,
            )
            .await?;

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn handle_set_single_local_value_with_single_record_lock(
        &self,
        record_lock: &StorageManagerRecordLockGuard,
        subkey: ValueSubkey,
        value: Arc<SignedValueData>,
    ) -> VeilidAPIResult<()> {
        let opaque_record_key = record_lock.record();

        // Remove any offline writes to this subkey since we're rewriting it
        {
            let mut inner = self.inner.lock();
            self.remove_offline_subkey_write_inner(&mut inner, &opaque_record_key, subkey);
        }

        // Write subkey to local store
        let local_record_store = self.get_local_record_store()?;
        local_record_store
            .set_single_subkey(
                &opaque_record_key,
                subkey,
                value.clone(),
                InboundWatchUpdateMode::NoUpdate,
                CommitActionFlushMode::Immediate,
            )
            .await?;

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    #[expect(dead_code)]
    pub(super) async fn handle_set_local_values_with_single_record_lock(
        &self,
        record_lock: &StorageManagerRecordLockGuard,
        subkey_values: SubkeyValueList,
    ) -> VeilidAPIResult<()> {
        let opaque_record_key = record_lock.record();

        // Remove any offline writes to this subkey since we're rewriting it
        {
            let mut inner = self.inner.lock();
            for subkey in subkey_values.iter().map(|x| x.0) {
                self.remove_offline_subkey_write_inner(&mut inner, &opaque_record_key, subkey);
            }
        }

        // Write subkey to local store
        let local_record_store = self.get_local_record_store()?;
        local_record_store
            .set_subkeys_single_record(
                &opaque_record_key,
                &subkey_values,
                InboundWatchUpdateMode::NoUpdate,
                CommitActionFlushMode::Immediate,
            )
            .await?;

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn handle_set_local_values_with_multiple_records_lock(
        &self,
        records_lock: &StorageManagerRecordsLockGuard,
        keys_and_subkeys: RecordSubkeyValueList,
    ) -> VeilidAPIResult<()> {
        let records = records_lock.records().into_iter().collect::<BTreeSet<_>>();
        for x in keys_and_subkeys.iter() {
            if !records.contains(&x.0) {
                apibail_internal!("invalid records lock")
            }
        }

        // See if this new data supercedes any offline subkey writes
        {
            let mut inner = self.inner.lock();
            for (opaque_record_key, subkey_values) in keys_and_subkeys.iter() {
                for subkey in subkey_values.iter().map(|x| x.0) {
                    self.remove_offline_subkey_write_inner(&mut inner, opaque_record_key, subkey);
                }
            }
        }

        // Write subkey to local store
        let local_record_store = self.get_local_record_store()?;
        local_record_store
            .set_subkeys_multiple_records(
                &keys_and_subkeys,
                InboundWatchUpdateMode::NoUpdate,
                CommitActionFlushMode::Immediate,
            )
            .await?;

        Ok(())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn handle_inspect_local_values_with_peek_lock(
        &self,
        peek_lock: &StorageManagerPeekLockGuard,
        subkeys: ValueSubkeyRangeSet,
        want_descriptor: bool,
    ) -> VeilidAPIResult<InspectResult> {
        let opaque_record_key = peek_lock.record();

        // See if it's in the local record store
        let local_record_store = self.get_local_record_store()?;

        if let Some(inspect_result) = local_record_store
            .inspect_record(&opaque_record_key, &subkeys, want_descriptor)
            .await?
        {
            return Ok(inspect_result);
        }

        Ok(InspectResult::default())
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) fn get_value_nodes(
        &self,
        opaque_record_key: &OpaqueRecordKey,
    ) -> VeilidAPIResult<Option<Vec<NodeRef>>> {
        // Get local record store
        let local_record_store = self.get_local_record_store()?;

        // Get routing table to see if we still know about these nodes
        let routing_table = self.routing_table();

        let opt_value_nodes = local_record_store.peek_record(opaque_record_key, |r| {
            let d = r.detail();
            d.nodes
                .keys()
                .cloned()
                .filter_map(|nr| routing_table.lookup_node_ref(nr).ok().flatten())
                .collect()
        });

        Ok(opt_value_nodes)
    }
}
