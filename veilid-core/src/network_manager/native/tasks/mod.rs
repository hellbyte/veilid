mod network_interfaces_task;
mod update_dial_info;
mod upnp_task;

use super::*;

impl Network {
    pub fn setup_tasks(&self) {
        // Set update network class tick task
        let this = self.clone();
        self.update_dial_info_task.set_routine(move |s, l, t| {
            let this = this.clone();
            Box::pin(async move {
                this.update_dial_info_task_routine(s, Timestamp::new(l), Timestamp::new(t))
                    .await
            })
        });

        // Set network interfaces tick task
        let this = self.clone();
        self.network_interfaces_task.set_routine(move |s, l, t| {
            let this = this.clone();
            Box::pin(async move {
                this.network_interfaces_task_routine(s, Timestamp::new(l), Timestamp::new(t))
                    .await
            })
        });

        // Set upnp tick task
        {
            let this = self.clone();
            self.upnp_task.set_routine(move |s, l, t| {
                let this = this.clone();
                Box::pin(async move {
                    this.upnp_task_routine(s, Timestamp::new(l), Timestamp::new(t))
                        .await
                })
            });
        }
    }

    // Determine if we need to check for public dialinfo
    fn wants_update_dial_info_tick(&self) -> bool {
        let routing_table = self.routing_table();

        let routing_domain_state =
            routing_table.routing_domain_state(RoutingDomain::PublicInternet);

        let (state_wants_dial_info, state_is_publishable) = match routing_domain_state {
            RoutingDomainState::Invalid | RoutingDomainState::Unusable => {
                // Never tick if we haven't set up the network or the network is not usable
                return false;
            }
            RoutingDomainState::NeedsDialInfoConfirmation => {
                // Still need to confirm dial info
                (true, false)
            }
            RoutingDomainState::NeedsRelays { relay_status: _ }
            | RoutingDomainState::ReadyToPublish { relay_status: _ } => {
                // Already have confirmed dialinfo
                (false, true)
            }
        };

        let current_peer_info = routing_table.get_current_peer_info(RoutingDomain::PublicInternet);

        let needs_update_dial_info = self.needs_update_dial_info();
        if needs_update_dial_info
            || state_wants_dial_info
            || (state_is_publishable
                && !current_peer_info.node_info().has_dial_info()
                && self.inner.lock().next_outbound_only_dial_info_check <= Timestamp::now())
        {
            let live_entry_counts = routing_table.cached_live_entry_counts();

            // Bootstrap needs to have gotten us connectivity nodes
            let mut has_at_least_two = true;
            for ck in VALID_CRYPTO_KINDS {
                if live_entry_counts
                    .connectivity_capabilities
                    .get(&(RoutingDomain::PublicInternet, ck))
                    .copied()
                    .unwrap_or_default()
                    < MIN_BOOTSTRAP_CONNECTIVITY_PEERS
                {
                    has_at_least_two = false;
                    break;
                }
            }

            has_at_least_two
        } else {
            false
        }
    }

    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "net", name = "Network::tick", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    pub async fn tick(&self) -> EyreResult<()> {
        let Ok(_guard) = self.startup_lock.enter() else {
            veilid_log!(self debug "ignoring 'Network::tick' due to not started up");
            return Ok(());
        };

        // Ignore this tick if we need to restart
        if self.needs_restart() {
            return Ok(());
        }

        let (upnp, require_inbound_relay) = {
            let config = self.config();
            (
                config.network.upnp,
                config.network.privacy.require_inbound_relay,
            )
        };

        if require_inbound_relay {
            // Configured to only use relays for inbound connections.
            // This implicitly turns off address detection and upnp.
            return Ok(());
        }

        // If we need to figure out our network class, tick the task for it
        if self.resolved_detect_address_changes() {
            // Check our network interfaces to see if they have changed
            self.network_interfaces_task.tick().await?;

            if self.wants_update_dial_info_tick() {
                self.update_dial_info_task.tick().await?;
            }
        }

        // If we need to tick upnp, do it
        if upnp {
            self.upnp_task.tick().await?;
        }

        Ok(())
    }

    pub async fn cancel_tasks(&self) {
        veilid_log!(self debug "stopping upnp task");
        if let Err(e) = self.upnp_task.stop().await {
            veilid_log!(self warn "upnp_task not stopped: {}", e);
        }
        veilid_log!(self debug "stopping network interfaces task");
        if let Err(e) = self.network_interfaces_task.stop().await {
            veilid_log!(self warn "network_interfaces_task not stopped: {}", e);
        }
        veilid_log!(self debug "stopping update network class task");
        if let Err(e) = self.update_dial_info_task.stop().await {
            veilid_log!(self warn "update_dial_info_task not stopped: {}", e);
        }
    }
}
