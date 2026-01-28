use crate::{network_manager::StartupDisposition, *};
use routing_table::RoutingTableHealth;

impl_veilid_log_facility!("attach");

const TICK_INTERVAL_MSEC: u32 = 1000;
const ATTACHMENT_MAINTAINER_INTERVAL_MSEC: u32 = 1000;
const BIND_WAIT_DELAY: TimestampDuration = TimestampDuration::new_ms(10000);

#[derive(Debug, Clone)]
pub struct AttachmentManagerStartupContext {
    pub initialize_lock: Arc<StartupLock>,
    pub attachment_lock: Arc<StartupLock>,
}
impl AttachmentManagerStartupContext {
    pub fn new() -> Self {
        Self {
            initialize_lock: Arc::new(StartupLock::new()),
            attachment_lock: Arc::new(StartupLock::new()),
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
    started_ts: Timestamp,
    attach_ts: Option<Timestamp>,
    bind_retry_ts: Option<Timestamp>,
    last_tick_ts: Option<Timestamp>,
    tick_future: Option<PinBoxFutureStatic<()>>,
}

impl fmt::Debug for AttachmentManagerInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttachmentManagerInner")
            .field("attachment_state", &self.attachment_state)
            .field("last_routing_table_health", &self.last_routing_table_health)
            .field("maintain_peers", &self.maintain_peers)
            .field("started_ts", &self.started_ts)
            .field("attach_ts", &self.attach_ts)
            .field("bind_retry_ts", &self.bind_retry_ts)
            .field("last_tick_ts", &self.last_tick_ts)
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
            started_ts: Timestamp::now(),
            attach_ts: None,
            last_tick_ts: None,
            bind_retry_ts: None,
            tick_future: None,
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

    fn log_facilities_impl(&self) -> VeilidComponentLogFacilities {
        VeilidComponentLogFacilities::new()
            .with_facility(VeilidComponentLogFacility::try_new("attach").unwrap())
    }

    #[cfg_attr(feature = "instrument", instrument(level = "debug", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key())))]
    #[allow(clippy::unused_async)]
    pub async fn init_async(&self) -> EyreResult<()> {
        let guard = self.startup_context.initialize_lock.startup()?;
        guard.success();
        Ok(())
    }

    #[cfg_attr(feature = "instrument", instrument(level = "debug", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key())))]
    #[allow(clippy::unused_async)]
    pub async fn post_init_async(&self) -> EyreResult<()> {
        let registry = self.registry();

        veilid_log!(self debug "starting attachment maintainer task");
        impl_setup_task_async!(
            self,
            Self,
            attachment_maintainer_task,
            attachment_maintainer_task_routine
        );

        let mut inner = self.inner.lock();

        // Let the attachment maintainer run
        let guard = self.startup_context.attachment_lock.startup()?;
        guard.success();

        // Create top level tick interval
        let tick_future = interval(
            "attachment maintainer tick",
            TICK_INTERVAL_MSEC,
            true,
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

        // Keep ticker, drop this to stop the ticker
        inner.tick_future = Some(tick_future);

        Ok(())
    }

    #[cfg_attr(feature = "instrument", instrument(level = "debug", skip_all, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub async fn pre_terminate_async(&self) {
        // Start attachment shutdown
        let guard = self
            .startup_context
            .attachment_lock
            .shutdown()
            .await
            .expect_or_log("should be initialized");

        // Wait for detached state
        while !matches!(
            self.inner.lock().attachment_state,
            AttachmentState::Detached
        ) {
            sleep(100).await;
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

        // Attachent shutdown successful
        guard.success();
    }

    #[cfg_attr(feature = "instrument", instrument(level = "debug", skip_all, fields(__VEILID_LOG_KEY = self.log_key())))]
    pub async fn terminate_async(&self) {
        let guard = self
            .startup_context
            .initialize_lock
            .shutdown()
            .await
            .expect_or_log("should be initialized");

        // Shutdown successful
        guard.success();
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip_all, fields(__VEILID_LOG_KEY = self.log_key())))]
    #[allow(clippy::unused_async)]
    pub async fn attach(&self) -> bool {
        let Ok(_guard) = self.startup_context.initialize_lock.enter() else {
            return false;
        };

        let mut inner = self.inner.lock();
        // If attaching is disabled (because we are terminating)
        // then just return now

        if !self.startup_context.attachment_lock.is_started() {
            return false;
        }
        let previous = inner.maintain_peers;
        inner.maintain_peers = true;

        previous != inner.maintain_peers
    }

    #[cfg_attr(feature = "instrument", instrument(level = "trace", skip_all, fields(__VEILID_LOG_KEY = self.log_key())))]
    #[allow(clippy::unused_async)]
    pub async fn detach(&self) -> bool {
        let Ok(_guard) = self.startup_context.initialize_lock.enter() else {
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
    #[cfg_attr(
        feature = "instrument",
        instrument(level = "trace", target = "stor", skip_all, err, fields(__VEILID_LOG_KEY = self.log_key()))
    )]
    async fn attachment_maintainer_task_routine(
        &self,
        _stop_token: StopToken,
        _last_ts: Timestamp,
        cur_ts: Timestamp,
    ) -> EyreResult<()> {
        // Get the state to process
        let (mut state, maintain_peers, bind_retry_waiting) = {
            let mut inner = self.inner.lock();

            // Check for bind wait retry timeout
            let bind_retry_waiting =
                if inner.bind_retry_ts.is_some() && inner.bind_retry_ts.unwrap() > cur_ts {
                    true
                } else {
                    inner.bind_retry_ts = None;
                    false
                };

            (
                inner.attachment_state,
                inner.maintain_peers,
                bind_retry_waiting,
            )
        };

        // Enter the attachment lock and return if we are shutting down
        let Ok(_guard) = self.startup_context.attachment_lock.enter() else {
            // Only shutdown is possible if we get here
            match state {
                AttachmentState::Detached | AttachmentState::Detaching => {
                    // ok
                }
                AttachmentState::Attaching
                | AttachmentState::AttachedWeak
                | AttachmentState::AttachedGood
                | AttachmentState::AttachedStrong
                | AttachmentState::FullyAttached
                | AttachmentState::OverAttached => {
                    veilid_log!(self debug "terminating attachment maintainer task");
                    self.update_non_attached_state(AttachmentState::Detaching);
                    self.inner.lock().attachment_state = AttachmentState::Detaching;

                    self.shutdown().await;

                    self.update_non_attached_state(AttachmentState::Detached);
                    self.inner.lock().attachment_state = AttachmentState::Detached;
                }
            }

            return Ok(());
        };

        // Process the attachment state machine
        loop {
            let (next_state, continue_loop) = match state {
                AttachmentState::Detached => {
                    if maintain_peers && !bind_retry_waiting {
                        veilid_log!(self debug "attachment starting");

                        match self.startup().await {
                            Err(err) => {
                                error!("attachment startup failed: {}", err);
                                (None, false)
                            }
                            Ok(StartupDisposition::BindRetry) => {
                                veilid_log!(self info "waiting for network to bind...");
                                self.inner.lock().bind_retry_ts =
                                    Some(Timestamp::now_non_decreasing().later(BIND_WAIT_DELAY));
                                (None, false)
                            }
                            Ok(StartupDisposition::Success) => {
                                veilid_log!(self debug "started maintaining peers");

                                self.update_non_attached_state(AttachmentState::Attaching);
                                (Some(AttachmentState::Attaching), false)
                            }
                        }
                    } else {
                        (None, false)
                    }
                }
                AttachmentState::Attaching
                | AttachmentState::AttachedWeak
                | AttachmentState::AttachedGood
                | AttachmentState::AttachedStrong
                | AttachmentState::FullyAttached
                | AttachmentState::OverAttached => {
                    if maintain_peers {
                        let network_manager = self.network_manager();
                        if network_manager.network_needs_restart() {
                            veilid_log!(self info "Restarting network");
                            self.update_non_attached_state(AttachmentState::Detaching);
                            (Some(AttachmentState::Detaching), true)
                        } else {
                            (self.update_attached_state(state), false)
                        }
                    } else {
                        veilid_log!(self debug "stopped maintaining peers");
                        (Some(AttachmentState::Detaching), true)
                    }
                }
                AttachmentState::Detaching => {
                    veilid_log!(self debug "shutting down attachment");
                    self.shutdown().await;

                    self.update_non_attached_state(AttachmentState::Detached);
                    (Some(AttachmentState::Detached), false)
                }
            };

            // Transition to next state if it changed
            if let Some(next_state) = next_state {
                let mut inner = self.inner.lock();
                inner.attachment_state = next_state;
                state = next_state;
            }

            // Loop again if we should process the next state immediately
            if !continue_loop {
                break;
            }
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
        if let Err(e) = rpc_processor.startup() {
            network_manager.shutdown().await;
            return Err(e);
        }

        // Startup routing table
        let routing_table = self.routing_table();
        if let Err(e) = routing_table.startup() {
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
            >= TryInto::<usize>::try_into(config.limit_over_attached).unwrap_or_log()
        {
            return AttachmentState::OverAttached;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_fully_attached).unwrap_or_log()
        {
            return AttachmentState::FullyAttached;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_attached_strong).unwrap_or_log()
        {
            return AttachmentState::AttachedStrong;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_attached_good).unwrap_or_log()
        {
            return AttachmentState::AttachedGood;
        }
        if health.reliable_entry_count
            >= TryInto::<usize>::try_into(config.limit_attached_weak).unwrap_or_log()
            || health.unreliable_entry_count
                >= TryInto::<usize>::try_into(config.limit_attached_weak).unwrap_or_log()
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
