//! Debuggable Async Locks
use super::*;

mod async_mutex;
mod async_rw_lock;
mod async_semaphore;
mod async_tag_lock;
#[cfg(feature = "debug-locks")]
mod debug_locks;
#[cfg(all(feature = "debug-locks-detect", feature = "debug-locks"))]
mod debug_locks_detect;

pub use async_mutex::*;
pub use async_rw_lock::*;
pub use async_semaphore::*;
pub use async_tag_lock::*;

#[cfg(feature = "debug-locks")]
use debug_locks::*;
#[cfg(all(feature = "debug-locks-detect", feature = "debug-locks"))]
use debug_locks_detect::*;

pub const DEBUG_LOCKS_DURATION_MS: u32 = 30000;
