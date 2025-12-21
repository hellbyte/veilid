use crate::{network_manager::StartupDisposition, *};
use routing_table::RoutingTableHealth;

impl_veilid_log_facility!("attach");

const TICK_INTERVAL_MSEC: u32 = 1000;
const ATTACHMENT_MAINTAINER_INTERVAL_MSEC: u32 = 1000;
const BIND_WAIT_DELAY_MSEC: u32 = 10000;

#[derive(Debug, Clone)]
pub struct AttachmentManagerStartupContext {
    pub startup_lock: Arc<StartupLock>,
}
impl AttachmentManagerStartupContext {
    pub fn new() -> Self {
        Self {
            startup_lock: Arc::new(StartupLock::new()),
        }
    }
}
impl Default for AttachmentManagerStartupContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Event sent every second while veilid-core is initialized
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TickEvent {
    pub last_tick_ts: Option<Timestamp>,
    pub cur_tick_ts: Timestamp,
}

struct AttachmentManagerInner {
    attachment_state: AttachmentState,
    last_routing_table_health: Option<Arc<RoutingTableHealth>>,
    maintain_peers: bool,
    attach_enabled: bool,
    started_ts: Timestamp,
    attach_ts: Option<Timestamp>,
    last_tick_ts: Option<Timestamp>,
    tick_future: Option<PinBoxFutureStatic<()>>,
    eventual_termination: Option<EventualValue<()>>,
}

impl fmt::Debug for AttachmentManagerInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttachmentManagerInner")
            .field("attachment_state", &self.attachment_state)
            .field("last_routing_table_health", &self.last_routing_table_health)
            .field("maintain_peers", &self.maintain_peers)
            .field("attach_enabled", &self.attach_enabled)
            .field("started_ts", &self.started_ts)
            .field("attach_ts", &self.attach_ts)
            .field("last_tick_ts", &self.last_tick_ts)
            .field("eventual_termination", &self.eventual_termination)
            .finish()
    }
}

pub struct AttachmentManager {
    registry: VeilidComponentRegistry,
    inner: Mutex<AttachmentManagerInner>,
    startup_context: AttachmentManagerStartupContext,
    attachment_maintainer_task: TickTask<EyreReport>,
}

impl fmt::Debug for AttachmentManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttachmentManager")
            // .field("registry", &self.registry)
            .field("inner", &self.inner)
            .field("startup_context", &self.startup_context)
            // .field("attachment_maintainer_task", &self.attachment_maintainer_task)
            .finish()
    }
}

impl_veilid_component!(AttachmentManager);

impl AttachmentManager {
    fn new_inner() -> AttachmentManagerInner {
        AttachmentManagerInner {
            attachment_state: AttachmentState::Detached,
            last_routing_table_health: None,
            maintain_peers: false,
            attach_enabled: false,
            started_ts: Timestamp::now(),
            attach_ts: None,
            last_tick_ts: None,
            tick_future: None,
            eventual_termination: None,
        }
    }
    pub fn new(
        registry: VeilidComponentRegistry,
        startup_context: AttachmentManagerStartupContext,
    ) -> Self {
        Self {
            registry,
            inner: Mutex::new(Self::new_inner()),
            startup_context,
            attachment_maintainer_task: TickTask::new_ms(
                "attachment_maintainer_task",
                ATTACHMENT_MAINTAINER_INTERVAL_MSEC,
            ),
        }
    }

    pub fn is_attached(&self) -> bool {
        self.inner.lock().attachment_state.is_attached()
    }

    #[allow(dead_code)]
    pub fn is_detached(&self) -> bool {
        self.inner.lock().attachment_state.is_detached()
    }

    #[allow(dead_code)]
    pub fn get_attach_timestamp(&self) -> Option<Timestamp> {
        self.inner.lock().attach_ts
    }

    #[instrument(level = "debug", skip_all, err)]
    pub async fn init_async(&self) -> EyreResult<()> {
        let guard = self.startup_context.startup_lock.startup()?;
        guard.success();
        Ok(())
    }

    #[instrument(level = "debug", skip_all, err)]
    pub async fn post_init_async(&self) -> EyreResult<()> {
        let registry = self.registry();

        veilid_log!(self debug "starting attachment maintainer task");
        impl_setup_task_async!(
            self,
            Self,
            attachment_maintainer_task,
            attachment_maintainer_task_routine
        );

        // Create top level tick interval
        let tick_future = interval(
            "attachment maintainer tick",
            TICK_INTERVAL_MSEC,
            move || {
                let registry = registry.clone();
                async move {
                    let this = registry.attachment_manager();
                    if let Err(e) = this.tick().await {
                        veilid_log!(this warn "attachment maintainer tick failed: {}", e);
                    }
                }
            },
        );

        {
            let mut inner = self.inner.lock();
            inner.tick_future = Some(tick_future);

            // Enable attachment now
            inner.attach_enabled = true;
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn pre_terminate_async(&self) {
        {
            let mut inner = self.inner.lock();
            // Disable attachment now
            // Will cause attachment maintainer to drive the state toward 'Detached'
            inner.attach_enabled = false;
        }

        // Wait for detached state
        while !matches!(
            self.inner.lock().attachment_state,
            AttachmentState::Detached
        ) {
            sleep(500).await;
        }

        // Stop ticker
        let tick_future = self.inner.lock().tick_future.take();
        if let Some(tick_future) = tick_future {
            tick_future.await;
        }

        // Stop background operations
        veilid_log!(self debug "stopping attachment maintainer task");
        if let Err(e) = self.attachment_maintainer_task.stop().await {
            veilid_log!(self warn "attachment_maintainer not stopped: {}", e);
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn terminate_async(&self) {
        let guard = self
            .startup_context
            .startup_lock
            .shutdown()
            .await
            .expect("should be initialized");

        // Shutdown successful
        guard.success();
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn attach(&self) -> bool {
        let Ok(_guard) = self.startup_context.startup_lock.enter() else {
            return false;
        };

        let mut inner = self.inner.lock();
        // If attaching is disabled (because we are terminating)
        // then just return now
        if !inner.attach_enabled {
            return false;
        }
        let previous = inner.maintain_peers;
        inner.maintain_peers = true;

        previous != inner.maintain_peers
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn detach(&self) -> bool {
        let Ok(_guard) = self.startup_context.startup_lock.enter() else {
            return false;
        };

        {
            let mut inner = self.inner.lock();
            let previous = inner.maintain_peers;
            if !previous {
                // Already detached or detaching
                return false;
            }
            // Wants to be detached
            inner.maintain_peers = false;
        }

        true
    }

    /////////////////////////////////////////////////////////////////////////////

    async fn tick(&self) -> EyreResult<()> {
        let cur_tick_ts = Timestamp::now_non_decreasing();
        let last_tick_ts = {
            let mut inner = self.inner.lock();
            let last_tick_ts = inner.last_tick_ts;
            inner.last_tick_ts = Some(cur_tick_ts);
            last_tick_ts
        };

        // Log if we're seeing missed ticks
        if let Some(lag) = last_tick_ts.map(|x| cur_tick_ts.duration_since(x)) {
            if lag > TimestampDuration::new_ms(2 * (TICK_INTERVAL_MSEC as u64)) {
                veilid_log!(self debug "tick lag: {}", lag);
            }
        }

        // Tick our own ticktask for the attachment maintainer state machine
        self.attachment_maintainer_task.tick().await?;

        // Send a 'tick' event for the rest of the system to get ticks
        let event_bus = self.event_bus();
        event_bus.post(TickEvent {
            last_tick_ts,
            cur_tick_ts,
        })?;

        Ok(())
    }

    // Manage attachment state
    #[instrument(level = "trace", target = "stor", skip_all, err)]
    async fn attachment_maintainer_task_routine(
        &self,
        _stop_token: StopToken,
        _last_ts: Timestamp,
        _cur_ts: Timestamp,
    ) -> EyreResult<()> {
        let (state, maintain_peers, attach_enabled) = {
            let inner = self.inner.lock();
            (
                inner.attachment_state,
                inner.maintain_peers,
                inner.attach_enabled,
            )
        };

        let next_state = match state {
            AttachmentState::Detached => {
                if maintain_peers && attach_enabled {
                    veilid_log!(self debug "attachment starting");

                    match self.startup().await {
                        Err(err) => {
                            error!("attachment startup failed: {}", err);
                            None
                        }
                        Ok(StartupDisposition::BindRetry) => {
                            veilid_log!(self info "waiting for network to bind...");
                            sleep(BIND_WAIT_DELAY_MSEC).await;
                            None
                        }
                        Ok(StartupDisposition::Success) => {
                            veilid_log!(self debug "started maintaining peers");

                            self.update_non_attached_state(AttachmentState::Attaching);
                            Some(AttachmentState::Attaching)
                        }
                    }
                } else {
                    None
                }
            }
            AttachmentState::Attaching
            | AttachmentState::AttachedWeak
            | AttachmentState::AttachedGood
            | AttachmentState::AttachedStrong
            | AttachmentState::FullyAttached
            | AttachmentState::OverAttached => {
                if maintain_peers && attach_enabled {
                    let network_manager = self.network_manager();
                    if network_manager.network_needs_restart() {
                        veilid_log!(self info "Restarting network");
                        self.update_non_attached_state(AttachmentState::Detaching);
                        Some(AttachmentState::Detaching)
                    } else {
                        self.update_attached_state(state)
                    }
                } else {
                    veilid_log!(self debug "stopped maintaining peers");
                    Some(AttachmentState::Detaching)
                }
            }
            AttachmentState::Detaching => {
                veilid_log!(self debug "shutting down attachment");
                self.shutdown().await;

                self.update_non_attached_state(AttachmentState::Detached);
                Some(AttachmentState::Detached)
            }
        };

        // Transition to next state
        if let Some(next_state) = next_state {
            let mut inner = self.inner.lock();
            inner.attachment_state = next_state;
        }

        Ok(())
    }

    async fn startup(&self) -> EyreResult<StartupDisposition> {
        let rpc_processor = self.rpc_processor();
        let network_manager = self.network_manager();

        // Startup network manager
        let res = network_manager.startup().await?;
        match res {
            StartupDisposition::Success => {
                veilid_log!(self debug "NetworkManager startup success");
            }
            StartupDisposition::BindRetry => {
                veilid_log!(self debug "NetworkManager bind retry");
                return Ok(StartupDisposition::BindRetry);
            }
        }

        // Startup rpc processor
        if let Err(e) = rpc_processor.startup().await {
            network_manager.shutdown().await;
            return Err(e);
        }

        // Startup routing table
        let routing_table = self.routing_table();
        if let Err(e) = routing_table.startup().await {
            rpc_processor.shutdown().await;
            network_manager.shutdown().await;
            return Err(e);
        }

        // Inform api clients that things have changed
        veilid_log!(self trace "sending network state update to api clients");
        network_manager.send_network_update();

        Ok(StartupDisposition::Success)
    }

    async fn shutdown(&self) {
        let routing_table = self.routing_table();
        let rpc_processor = self.rpc_processor();
        let network_manager = self.network_manager();

        // Shutdown RoutingTable
        routing_table.shutdown().await;

        // Shutdown NetworkManager
        network_manager.shutdown().await;

        // Shutdown RPCProcessor
        rpc_processor.shutdown().await;

        // Send update
        veilid_log!(self debug "sending network state update to api clients");
        network_manager.send_network_update();
    }

    fn translate_routing_table_health(
        health: &RoutingTableHealth,
        config: &VeilidConfigRoutingTable,
    ) -> AttachmentState {
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_over_attached).unwrap()
        {
            return AttachmentState::OverAttached;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_fully_attached).unwrap()
        {
            return AttachmentState::FullyAttached;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_attached_strong).unwrap()
        {
            return AttachmentState::AttachedStrong;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_attached_good).unwrap()
        {
            return AttachmentState::AttachedGood;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_attached_weak).unwrap()
            || health.unreliable_entry_count
                >= TryInto::<usize>::try_into(config.limit_attached_weak).unwrap()
        {
            return AttachmentState::AttachedWeak;
        }
        AttachmentState::Attaching
    }

    /// Update attachment and network readiness state
    /// and possibly send a VeilidUpdate::Attachment.
    fn update_attached_state(
        &self,
        current_attachment_state: AttachmentState,
    ) -> Option<AttachmentState> {
        // update the routing table health
        let routing_table = self.network_manager().routing_table();
        let health = routing_table.get_routing_table_health();
        let (opt_update, opt_next_attachment_state) = {
            let mut inner = self.inner.lock();

            // Check if the routing table health is different
            if let Some(last_routing_table_health) = &inner.last_routing_table_health {
                // If things are the same, just return
                if last_routing_table_health.as_ref() == &health {
                    return None;
                }
            }

            // Swap in new health numbers
            let opt_previous_health = inner.last_routing_table_health.take();
            inner.last_routing_table_health = Some(Arc::new(health.clone()));

            // Calculate new attachment state
            let config = self.config();
            let routing_table_config = &config.network.routing_table;
            let next_attachment_state =
                AttachmentManager::translate_routing_table_health(&health, routing_table_config);

            // Send update if one of:
            // * the attachment state has changed
            // * routing domain readiness has changed
            // * this is our first routing table health check
            let send_update = current_attachment_state != next_attachment_state
                || opt_previous_health
                    .map(|x| {
                        x.public_internet_ready != health.public_internet_ready
                            || x.local_network_ready != health.local_network_ready
                    })
                    .unwrap_or(true);
            let opt_update = if send_update {
                Some(Self::get_veilid_state_inner(&inner))
            } else {
                None
            };
            let opt_next_attachment_state = if current_attachment_state != next_attachment_state {
                Some(next_attachment_state)
            } else {
                None
            };

            (opt_update, opt_next_attachment_state)
        };

        // Send the update outside of the lock
        if let Some(update) = opt_update {
            (self.update_callback())(VeilidUpdate::Attachment(update));
        }

        opt_next_attachment_state
    }

    fn update_non_attached_state(&self, current_attachment_state: AttachmentState) {
        let uptime;
        let attached_uptime;
        {
            let mut inner = self.inner.lock();

            // Clear routing table health so when we start measuring it we start from scratch
            inner.last_routing_table_health = None;

            // Set timestamps
            if current_attachment_state == AttachmentState::Attaching {
                inner.attach_ts = Some(Timestamp::now_non_decreasing());
            } else if current_attachment_state == AttachmentState::Detached {
                inner.attach_ts = None;
            } else if current_attachment_state == AttachmentState::Detaching {
                // ok
            } else {
                unreachable!("don't use this for attached states, use update_attached_state()");
            }

            let now = Timestamp::now_non_decreasing();
            uptime = now.duration_since(inner.started_ts);
            attached_uptime = inner.attach_ts.map(|ts| now.duration_since(ts));
        };

        // Send update
        (self.update_callback())(VeilidUpdate::Attachment(Box::new(VeilidStateAttachment {
            state: current_attachment_state,
            public_internet_ready: false,
            local_network_ready: false,
            uptime,
            attached_uptime,
        })))
    }

    fn get_veilid_state_inner(inner: &AttachmentManagerInner) -> Box<VeilidStateAttachment> {
        let now = Timestamp::now_non_decreasing();
        let uptime = now.duration_since(inner.started_ts);
        let attached_uptime = inner.attach_ts.map(|ts| now.duration_since(ts));

        Box::new(VeilidStateAttachment {
            state: inner.attachment_state,
            public_internet_ready: inner
                .last_routing_table_health
                .as_ref()
                .map(|x| x.public_internet_ready)
                .unwrap_or(false),
            local_network_ready: inner
                .last_routing_table_health
                .as_ref()
                .map(|x| x.local_network_ready)
                .unwrap_or(false),
            uptime,
            attached_uptime,
        })
    }

    pub fn get_veilid_state(&self) -> Box<VeilidStateAttachment> {
        let inner = self.inner.lock();
        Self::get_veilid_state_inner(&inner)
    }

    #[expect(dead_code)]
    pub fn get_attachment_state(&self) -> AttachmentState {
        self.inner.lock().attachment_state
    }

    #[expect(dead_code)]
    pub fn get_last_routing_table_health(&self) -> Option<Arc<RoutingTableHealth>> {
        self.inner.lock().last_routing_table_health.clone()
    }
}
