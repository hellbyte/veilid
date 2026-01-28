use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LockSense {
    Read,
    Write,
    TryRead,
    TryWrite,
}
impl LockSense {
    fn would_block(&self, earlier: LockSense) -> bool {
        match (earlier, self) {
            (LockSense::Read | LockSense::TryRead, LockSense::Read) => false,
            (LockSense::Read | LockSense::TryRead, LockSense::Write) => true,
            (LockSense::Read | LockSense::TryRead, LockSense::TryRead) => false,
            (LockSense::Read | LockSense::TryRead, LockSense::TryWrite) => false,
            (LockSense::Write | LockSense::TryWrite, LockSense::Read) => true,
            (LockSense::Write | LockSense::TryWrite, LockSense::Write) => true,
            (LockSense::Write | LockSense::TryWrite, LockSense::TryRead) => false,
            (LockSense::Write | LockSense::TryWrite, LockSense::TryWrite) => false,
        }
    }
}

impl fmt::Display for LockSense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LockSense::Read => "read",
                LockSense::Write => "write",
                LockSense::TryRead => "try_read",
                LockSense::TryWrite => "try_write",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct LaterLockStateKey {
    pub earlier_lock_sense: LockSense,
    pub later_lock_sense: LockSense,
}

#[derive(Debug, Clone)]
pub(super) struct LaterLockStateValue {
    pub earlier_lock_backtrace: backtrace::Backtrace,
    pub later_lock_backtrace: backtrace::Backtrace,
    pub task_id: AsyncTaskId,
}

#[derive(Default, Debug)]
pub(super) struct LaterLockState {
    pub entries: BTreeMap<LaterLockStateKey, LaterLockStateValue>,
}

impl LockIdContainer {
    pub(super) fn report_lock_inversion(
        &self,
        earlier_guard_state: &DebugGuardState,
        current_guard_state: &DebugGuardState,
        deadlock_combinations: &[(LaterLockStateKey, LaterLockStateValue)],
    ) {
        eprintln!("Lock inversion detected!");
        eprintln!(
            r#"First ordering:
    Earlier lock ({} task_id={:?}):
{}
    Later lock ({} task_id={:?}):
{}
"#,
            earlier_guard_state.sense,
            earlier_guard_state.task_id,
            indent::indent_all_by(8, earlier_guard_state.lock_backtrace.trim().to_string()),
            current_guard_state.sense,
            current_guard_state.task_id,
            indent::indent_all_by(8, current_guard_state.lock_backtrace.trim().to_string()),
        );
        for (n, entry) in deadlock_combinations.iter().enumerate() {
            eprintln!(
                r#"Second ordering #{}: (task_id={:?})
    Earlier lock ({}):
{}
    Later lock ({}): 
{}
"#,
                n,
                entry.1.task_id,
                entry.0.earlier_lock_sense,
                indent::indent_all_by(8, entry.1.earlier_lock_backtrace.trim().to_string()),
                entry.0.later_lock_sense,
                indent::indent_all_by(8, entry.1.later_lock_backtrace.trim().to_string()),
            );
        }

        // Don't panic, just exit
        std::process::exit(1);
    }
}

impl GuardIdContainer {
    pub(super) fn detect_lock_inversion(
        debug_states: &mut DebugState,
        current_guard_state: &DebugGuardState,
        lock_id_container: &LockIdContainer,
    ) {
        let task_id = AsyncTaskId::this();
        let current_raw_lock_id = lock_id_container.id();

        // Add the current-lock to the later-locks list for each earlier-lock that has an active guard at this point
        for earlier_guard_state in debug_states.guards.values() {
            let earlier_raw_lock_id = earlier_guard_state.lock_id_container.id();

            // Don't worry about single-lock deadlocks here, those are caught by the deadlock timeout and
            // are much more obvious. This way we also never add a lock to its own later-locks, which
            // can happen with read locks and semaphores pretty frequently.
            if earlier_raw_lock_id == current_raw_lock_id {
                continue;
            }

            // Only concern ourselves with guards that are in the same task as our own
            if earlier_guard_state.task_id != task_id {
                continue;
            }

            // If an earlier-lock is also in the current-lock's later-locks then there is a lock inversion problem
            let current_lock_state = debug_states.locks.get_mut(&current_raw_lock_id).unwrap();
            if let Some(current_lock_later_lock_state) =
                current_lock_state.later_locks.get(&earlier_raw_lock_id)
            {
                // Get lock senses for all guards involved (* = deadlock):

                // Scenario 1:
                // A --> B*     (Normal)
                // B ------> A* (Inverted)
                //
                // Scenario 2:
                // A ------> B* (Normal)
                // B --> A*     (Inverted)

                // Normal case has lock A before lock B
                let normal_guard_a = earlier_guard_state.sense;
                let normal_guard_b = current_guard_state.sense;

                // Iterate all recorded inverted cases
                let mut deadlock_combinations = vec![];
                for entry in current_lock_later_lock_state.entries.iter() {
                    // In inverted case, lock B comes before lock A
                    let inverted_guard_b = entry.0.earlier_lock_sense;
                    let inverted_guard_a = entry.0.later_lock_sense;

                    // If B* can lock B while A* locks A or vice versa, it's a deadlock
                    if normal_guard_b.would_block(inverted_guard_b)
                        && inverted_guard_a.would_block(normal_guard_a)
                    {
                        deadlock_combinations.push((entry.0.clone(), entry.1.clone()));
                    }
                }

                // Only report if we have a combination that would actually deadlock
                if !deadlock_combinations.is_empty() {
                    lock_id_container.report_lock_inversion(
                        earlier_guard_state,
                        &current_guard_state,
                        &deadlock_combinations,
                    );
                }
            }

            // Add current-lock to earlier-lock's later-lock state
            let earlier_lock_state = debug_states.locks.get_mut(&earlier_raw_lock_id).unwrap();
            let earlier_lock_later_lock_state = earlier_lock_state
                .later_locks
                .entry(current_raw_lock_id)
                .or_default();

            let earlier_lock_later_lock_state_key = LaterLockStateKey {
                earlier_lock_sense: earlier_guard_state.sense,
                later_lock_sense: current_guard_state.sense,
            };
            if !earlier_lock_later_lock_state
                .entries
                .contains_key(&earlier_lock_later_lock_state_key)
            {
                earlier_lock_later_lock_state.entries.insert(
                    earlier_lock_later_lock_state_key,
                    LaterLockStateValue {
                        earlier_lock_backtrace: earlier_guard_state.lock_backtrace.clone(),
                        later_lock_backtrace: current_guard_state.lock_backtrace.clone(),
                        task_id,
                    },
                );
            }
        }
    }
}
