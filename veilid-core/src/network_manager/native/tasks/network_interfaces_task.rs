use super::*;

impl Network {
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "net", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub(super) async fn network_interfaces_task_routine(
        &self,
        stop_token: StopToken,
        _l: Timestamp,
        _t: Timestamp,
    ) -> EyreResult<()> {
        // Network lock ensures only one task operating on the low level network state
        // can happen at the same time. Try lock is here to give preference to other longer
        // running processes like update_dial_info_task.
        let _guard = match self.network_task_lock.try_lock() {
            Some(v) => v,
            None => {
                // If we can't get the lock right now, then
                return Ok(());
            }
        };

        self.tick_network_state(stop_token).await?;

        Ok(())
    }

    // See if our interface addresses have changed, if so redo public dial info if necessary
    async fn tick_network_state(&self, _stop_token: StopToken) -> EyreResult<bool> {
        let last_network_state = self.last_network_state().unwrap_or_log();
        let new_network_state = match self.refresh_network_state().await {
            Ok(Some(v)) => v,
            Ok(None) => {
                // Nothing has changed
                return Ok(false);
            }
            Err(e) => {
                veilid_log!(self debug "Skipping network state update: {}", e);
                return Ok(false);
            }
        };

        // network state has changed
        let routing_table = self.routing_table();
        let resolved_detect_address_changes = self.resolved_detect_address_changes();

        // LocalNetwork
        let mut editor_local_network = routing_table.edit_local_network_routing_domain();
        editor_local_network.set_interface_addresses(
            new_network_state
                .interface_address_state
                .as_ref()
                .interface_addresses
                .clone(),
        );

        editor_local_network.setup_network(
            new_network_state.protocol_config.outbound,
            new_network_state.protocol_config.inbound,
            new_network_state.protocol_config.family_local,
            new_network_state
                .protocol_config
                .local_network_capabilities
                .clone(),
            true,
        );
        editor_local_network.clear_dial_info_details(None, None);

        // PublicInternet
        let mut editor_public_internet = routing_table.edit_public_internet_routing_domain();

        let enable_global_changed = last_network_state.enable_ipv4_global
            != new_network_state.enable_ipv4_global
            || last_network_state.enable_ipv6_global != new_network_state.enable_ipv6_global;

        let confirmed_public_internet =
            !resolved_detect_address_changes || self.config().network.privacy.require_inbound_relay;

        editor_public_internet.set_interface_addresses(
            new_network_state
                .interface_address_state
                .as_ref()
                .interface_addresses
                .clone(),
        );

        editor_public_internet.setup_network(
            new_network_state.protocol_config.outbound,
            new_network_state.protocol_config.inbound,
            new_network_state.protocol_config.family_global,
            new_network_state
                .protocol_config
                .public_internet_capabilities
                .clone(),
            confirmed_public_internet,
        );

        if enable_global_changed {
            // If our address families or routability changed,
            // clear the relays to recalculate them and
            // erase all prior dial so we can re-detect it
            editor_public_internet.set_relays(vec![]);
            editor_public_internet.clear_dial_info_details(None, None);
        }

        // Update protocols
        self.register_all_dial_info(&mut editor_public_internet, &mut editor_local_network)
            .await?;

        let local_network_changed = editor_local_network.commit(true).await;
        let public_internet_changed = editor_public_internet.commit(true).await;

        // Update local network now
        if local_network_changed {
            // No need to unpublish, just re-publish
            editor_local_network.publish();
        }

        // If any of the new addresses were PublicInternet addresses, re-run public dial info check
        if public_internet_changed {
            // Unpublish and then let dial info detection run and publish there if appropriate
            editor_public_internet.unpublish();
            self.trigger_update_dial_info(RoutingDomain::PublicInternet);
        }

        Ok(local_network_changed || public_internet_changed)
    }
}
