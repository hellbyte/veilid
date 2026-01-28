use super::*;
use std::sync::LazyLock;

pub(super) static DEBUG_STATES: LazyLock<Arc<Mutex<DebugState>>> =
    LazyLock::new(|| Default::default());

#[derive(Debug, Default)]
pub(super) struct DebugState {
    pub next_lock_id: usize,
    pub next_guard_id: usize,
    pub locks: HashMap<RawLockId, DebugLockState>,
    pub guards: HashMap<RawGuardId, DebugGuardState>,
}

#[derive(Default, Debug)]
pub(super) struct DebugLockState {
    pub create_backtrace: backtrace::Backtrace,
    pub active_guards: HashSet<RawGuardId>,
    #[cfg(feature = "debug-locks-detect")]
    pub later_locks: HashMap<RawLockId, LaterLockState>,
}

impl DebugLockState {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
pub(super) struct DebugGuardState {
    pub lock_id_container: LockIdContainer,
    pub lock_backtrace: backtrace::Backtrace,
    #[cfg(feature = "debug-locks-detect")]
    pub sense: LockSense,
    #[cfg(feature = "debug-locks-detect")]
    pub task_id: AsyncTaskId,
}

impl DebugGuardState {
    pub fn new(
        lock_id_container: LockIdContainer,
        #[cfg(feature = "debug-locks-detect")] sense: LockSense,
    ) -> Self {
        Self {
            lock_id_container,
            lock_backtrace: backtrace::Backtrace::new_unresolved(),
            #[cfg(feature = "debug-locks-detect")]
            sense,
            #[cfg(feature = "debug-locks-detect")]
            task_id: AsyncTaskId::this(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RawLockId(usize);

#[derive(Debug)]
struct LockIdContainerInner {
    raw_lock_id: RawLockId,
}

impl Drop for LockIdContainerInner {
    fn drop(&mut self) {
        let mut debug_states = DEBUG_STATES.lock();
        debug_states.locks.remove(&self.raw_lock_id);
    }
}

#[derive(Clone, Debug)]
pub(super) struct LockIdContainer {
    inner: Arc<LockIdContainerInner>,
}
impl LockIdContainer {
    pub fn id(&self) -> RawLockId {
        self.inner.raw_lock_id
    }

    pub fn next() -> LockIdContainer {
        let mut debug_states = DEBUG_STATES.lock();
        let raw_lock_id = RawLockId({
            let next_lock_id = debug_states.next_lock_id;
            debug_states.next_lock_id += 1;
            next_lock_id
        });
        debug_states
            .locks
            .insert(raw_lock_id, DebugLockState::new());
        LockIdContainer {
            inner: Arc::new(LockIdContainerInner { raw_lock_id }),
        }
    }

    pub fn report_deadlock(&self, desc: &str) -> ! {
        let mut debug_states = DEBUG_STATES.lock();
        let debug_lock_state = debug_states.locks.remove(&self.id()).unwrap();
        let active_guard_backtraces = debug_lock_state
            .active_guards
            .into_iter()
            .map(|gid| {
                let debug_guard_state = debug_states.guards.remove(&gid).unwrap();
                debug_guard_state.lock_backtrace.trim()
            })
            .collect::<Vec<_>>();

        eprintln!(
            "Deadlock detected! ({})\n\nLock creation backtrace:\n{}\n\nActive guard backtraces:\n",
            desc,
            debug_lock_state.create_backtrace.trim(),
        );
        for (n, agb) in active_guard_backtraces.iter().enumerate() {
            eprintln!("{}:\n{}\n", n, indent::indent_all_by(4, agb.to_string()));
        }

        let report_backtrace = backtrace::Backtrace::new_unresolved().trim();
        eprintln!("Panic backtrace:\n{}", report_backtrace);

        // Don't panic, just exit
        std::process::exit(1);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RawGuardId(usize);

#[derive(Debug)]
pub struct GuardIdContainerInner {
    raw_guard_id: RawGuardId,
}

impl Drop for GuardIdContainerInner {
    fn drop(&mut self) {
        let debug_guard_state = {
            let mut debug_states = DEBUG_STATES.lock();
            let debug_guard_state = debug_states.guards.remove(&self.raw_guard_id).unwrap();
            debug_states
                .locks
                .get_mut(&debug_guard_state.lock_id_container.id())
                .unwrap()
                .active_guards
                .remove(&self.raw_guard_id);
            debug_guard_state
        };
        drop(debug_guard_state);
    }
}

#[derive(Clone, Debug)]
pub struct GuardIdContainer {
    inner: Arc<GuardIdContainerInner>,
}

impl GuardIdContainer {
    #[expect(dead_code)]
    pub fn id(&self) -> RawGuardId {
        self.inner.raw_guard_id
    }
    pub fn next(
        lock_id_container: LockIdContainer,
        #[cfg(feature = "debug-locks-detect")] sense: LockSense,
    ) -> GuardIdContainer {
        // Produce a new current-lock guard id
        let debug_states = &mut *DEBUG_STATES.lock();
        let current_raw_guard_id = RawGuardId({
            let next_guard_id = debug_states.next_guard_id;
            debug_states.next_guard_id += 1;
            next_guard_id
        });

        // Make a new guard state to associate with this guard id
        let current_guard_state = DebugGuardState::new(
            lock_id_container.clone(),
            #[cfg(feature = "debug-locks-detect")]
            sense,
        );

        #[cfg(feature = "debug-locks-detect")]
        // Check for lock inversion deadlock
        Self::detect_lock_inversion(debug_states, &current_guard_state, &lock_id_container);

        // And then add the current-lock guard to the the current-lock state
        let current_raw_lock_id = lock_id_container.id();
        let current_lock_state = debug_states.locks.get_mut(&current_raw_lock_id).unwrap();
        current_lock_state
            .active_guards
            .insert(current_raw_guard_id);
        debug_states
            .guards
            .insert(current_raw_guard_id, current_guard_state);

        // Return the current-lock guard
        GuardIdContainer {
            inner: Arc::new(GuardIdContainerInner {
                raw_guard_id: current_raw_guard_id,
            }),
        }
    }
}
