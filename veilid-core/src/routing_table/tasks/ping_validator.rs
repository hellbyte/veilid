use super::*;

use futures_util::FutureExt;

impl_veilid_log_facility!("rtab");

/// Keepalive pings are done occasionally to ensure holepunched public dialinfo
/// remains valid, as well as to make sure we remain in any relay node's routing table
const RELAY_KEEPALIVE_PING_INTERVAL: TimestampDuration = TimestampDuration::new_secs(10);

/// Keepalive pings are done for active watch nodes to make sure they are still there
const ACTIVE_WATCH_KEEPALIVE_PING_INTERVAL: TimestampDuration = TimestampDuration::new_secs(10);

/// Ping queue processing depth per validator
const MAX_PARALLEL_PINGS: usize = 8;

type PingValidatorFuture = PinBoxFutureStatic<Result<(), RPCError>>;

impl RoutingTable {
    // Task routine for PublicInternet status pings
    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), err))]
    pub async fn ping_validator_public_internet_task_routine(
        &self,
        stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let mut future_queue: VecDeque<PingValidatorFuture> = VecDeque::new();

        self.ping_validator(cur_ts, RoutingDomain::PublicInternet, &mut future_queue)?;

        self.process_ping_validation_queue("PublicInternet", stop_token, cur_ts, future_queue)
            .await;

        Ok(())
    }

    // Task routine for LocalNetwork status pings
    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), err))]
    pub async fn ping_validator_local_network_task_routine(
        &self,
        stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let mut future_queue: VecDeque<PingValidatorFuture> = VecDeque::new();

        self.ping_validator(cur_ts, RoutingDomain::LocalNetwork, &mut future_queue)?;

        self.process_ping_validation_queue("LocalNetwork", stop_token, cur_ts, future_queue)
            .await;

        Ok(())
    }

    // Task routine for PublicInternet relay keepalive pings
    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), err))]
    pub async fn ping_validator_public_internet_relay_task_routine(
        &self,
        stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let mut future_queue: VecDeque<PingValidatorFuture> = VecDeque::new();

        self.relay_keepalive_public_internet(cur_ts, &mut future_queue)
            .await?;

        self.process_ping_validation_queue("RelayKeepalive", stop_token, cur_ts, future_queue)
            .await;

        Ok(())
    }

    // Task routine for active watch keepalive pings
    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip(self), err))]
    pub async fn ping_validator_active_watch_task_routine(
        &self,
        stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let mut future_queue: VecDeque<PingValidatorFuture> = VecDeque::new();

        self.active_watches_keepalive_public_internet(cur_ts, &mut future_queue)?;

        self.process_ping_validation_queue("WatchKeepalive", stop_token, cur_ts, future_queue)
            .await;

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Ping the relay to keep it alive, over every protocol it is relaying for us
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", skip(self, futurequeue), err)
    )]
    async fn relay_keepalive_public_internet(
        &self,
        cur_ts: Timestamp,
        futurequeue: &mut VecDeque<PingValidatorFuture>,
    ) -> EyreResult<()> {
        // Iterate the PublicInternet relays
        let relays_and_states = self.relays_and_states(RoutingDomain::PublicInternet);
        let mut state_updates = Vec::new();
        for (relay, mut relay_state) in relays_and_states {
            let relay_needs_keepalive =
                cur_ts.duration_since(relay_state.last_keepalive) >= RELAY_KEEPALIVE_PING_INTERVAL;

            if !relay_needs_keepalive {
                continue;
            }

            // Enqueue the pings
            for relay_ping in relay.pings.clone() {
                futurequeue.push_back(
                    async move {
                        let rpc_processor = relay_ping.node_ref.rpc_processor();
                        veilid_log!(rpc_processor trace "--> PublicInternet Relay ping to {:?}", relay_ping.node_ref);
                        let _ = Box::pin(rpc_processor
                            .rpc_call_status(Destination::direct(relay_ping.node_ref, None)))
                            .await?;
                        Ok(())
                    }
                    .boxed(),
                );
            }

            // Say we're doing this keepalive now
            relay_state.last_keepalive = cur_ts;
            state_updates.push((relay, relay_state));
        }

        // Update the relay keepalive timestamp on the routing domain
        if !state_updates.is_empty() {
            let mut editor = self.edit_public_internet_routing_domain();
            for (relay, state) in state_updates {
                editor.set_relay_state(relay, state);
            }
            editor.commit(false).await;
        }

        Ok(())
    }

    // Ping the active watch nodes to ensure they are still there
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", skip(self, futurequeue), err)
    )]
    fn active_watches_keepalive_public_internet(
        &self,
        cur_ts: Timestamp,
        futurequeue: &mut VecDeque<PingValidatorFuture>,
    ) -> EyreResult<()> {
        let watches_need_keepalive = {
            let mut inner = self.inner.write();
            let need = inner
                .opt_active_watch_keepalive_ts
                .map(|kts| cur_ts.duration_since(kts) >= ACTIVE_WATCH_KEEPALIVE_PING_INTERVAL)
                .unwrap_or(true);
            if need {
                inner.opt_active_watch_keepalive_ts = Some(cur_ts);
            }
            need
        };

        if !watches_need_keepalive {
            return Ok(());
        }

        // Get all the active watches from the storage manager
        let watch_destinations = self.storage_manager().get_outbound_watch_nodes();

        for watch_destination in watch_destinations {
            let registry = self.registry();
            futurequeue.push_back(
                async move {
                    let rpc_processor = registry.rpc_processor();
                    veilid_log!(rpc_processor trace "--> Watch Keepalive ping to {:?}", watch_destination);
                    let _ = Box::pin(rpc_processor.rpc_call_status(watch_destination)).await?;
                    Ok(())
                }
                .boxed(),
            );
        }
        Ok(())
    }

    // Ping each node in the routing table if they need to be pinged
    // to determine their reliability
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", skip(self, futurequeue), err)
    )]
    fn ping_validator(
        &self,
        cur_ts: Timestamp,
        routing_domain: RoutingDomain,
        futurequeue: &mut VecDeque<PingValidatorFuture>,
    ) -> EyreResult<()> {
        // Get all nodes needing pings in the chosen routing domain
        let node_refs = self.get_nodes_needing_ping(routing_domain, cur_ts);

        // Just do a single ping with the best protocol for all the other nodes to check for liveness
        for nr in node_refs {
            let nr = nr.with_sequencing(Sequencing::PreferOrdered);

            futurequeue.push_back(
                async move {
                    #[cfg(feature = "verbose-tracing")]
                    veilid_log!(nr debug "--> {:?} Validator ping to {:?}", routing_domain, nr);
                    let rpc_processor = nr.rpc_processor();
                    let _ = Box::pin(rpc_processor.rpc_call_status(Destination::direct(nr, None)))
                        .await?;
                    Ok(())
                }
                .boxed(),
            );
        }

        Ok(())
    }

    // Common handler for running ping validations in a batch
    async fn process_ping_validation_queue(
        &self,
        name: &str,
        stop_token: StopToken,
        cur_ts: Timestamp,
        future_queue: VecDeque<PingValidatorFuture>,
    ) {
        let count = future_queue.len();
        if count == 0 {
            return;
        }
        veilid_log!(self debug target:"network_result", "[{}] Ping validation queue: {} remaining", name, count);

        let atomic_count = AtomicUsize::new(count);
        let _ = process_batched_future_queue_result(future_queue, MAX_PARALLEL_PINGS, stop_token, |res| {
            if let Err(e) = res {
                veilid_log!(self debug "[{}] Error performing status ping: {}", name, e);
            }
            let remaining = atomic_count.fetch_sub(1, Ordering::AcqRel) - 1;
            if remaining > 0 {
                veilid_log!(self debug target:"network_result", "[{}] Ping validation queue: {} remaining", name, remaining);
            }
            Result::<(),()>::Ok(())
        })
        .await;
        let done_ts = Timestamp::now();
        veilid_log!(self debug
            "[{}] Ping validation queue finished {} pings in {}",
            name,
            count,
            done_ts.duration_since(cur_ts)
        );
    }
}
