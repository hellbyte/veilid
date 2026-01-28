//! AsyncRwLock
use super::*;

use async_lock::RwLock as InnerAsyncRwLock;
use async_lock::RwLockReadGuard as InnerAsyncRwLockReadGuard;
use async_lock::RwLockReadGuardArc as InnerAsyncRwLockReadGuardArc;
use async_lock::RwLockWriteGuard as InnerAsyncRwLockWriteGuard;
use async_lock::RwLockWriteGuardArc as InnerAsyncRwLockWriteGuardArc;

#[derive(Debug)]
pub struct AsyncRwLock<T>
where
    T: ?Sized,
{
    inner: Arc<InnerAsyncRwLock<T>>,
    #[cfg(feature = "debug-locks")]
    lock_id_container: LockIdContainer,
}

impl<T> AsyncRwLock<T> {
    pub fn new(t: T) -> AsyncRwLock<T> {
        AsyncRwLock {
            inner: Arc::new(InnerAsyncRwLock::new(t)),
            #[cfg(feature = "debug-locks")]
            lock_id_container: LockIdContainer::next(),
        }
    }

    #[inline]
    pub async fn read_arc(self: &Arc<Self>) -> AsyncRwLockReadGuardArc<T> {
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let inner = match timeout(DEBUG_LOCKS_DURATION_MS, self.inner.read_arc()).await {
                    Ok(v) => v,
                    Err(_) => {
                        self.lock_id_container.report_deadlock("AsyncRwLock::read_arc deadlock");
                    }
                };
            } else {
                let inner = self.inner.read_arc().await;
            }
        }

        AsyncRwLockReadGuardArc {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::Read,
            ),
        }
    }

    #[inline]
    #[must_use]
    pub fn try_read_arc(self: &Arc<Self>) -> Option<AsyncRwLockReadGuardArc<T>> {
        let inner = self.inner.try_read_arc()?;

        let out = AsyncRwLockReadGuardArc {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::TryRead,
            ),
        };

        Some(out)
    }
}

impl<T: ?Sized> AsyncRwLock<T> {
    #[inline]
    #[must_use]
    pub fn try_read(&self) -> Option<AsyncRwLockReadGuard<'_, T>> {
        let inner = self.inner.try_read()?;

        let out = AsyncRwLockReadGuard {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::TryRead,
            ),
        };

        Some(out)
    }

    #[inline]
    pub async fn read(&self) -> AsyncRwLockReadGuard<'_, T> {
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let inner = match timeout(DEBUG_LOCKS_DURATION_MS, self.inner.read()).await {
                    Ok(v) => v,
                    Err(_) => {
                        self.lock_id_container.report_deadlock("AsyncRwLock::read deadlock");
                    }
                };
            } else {
                let inner = self.inner.read().await;
            }
        }

        AsyncRwLockReadGuard {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::Read,
            ),
        }
    }

    #[inline]
    #[must_use]
    pub fn try_write(&self) -> Option<AsyncRwLockWriteGuard<'_, T>> {
        let inner = self.inner.try_write()?;

        let out = AsyncRwLockWriteGuard {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::TryWrite,
            ),
        };

        Some(out)
    }

    #[inline]
    pub async fn write(&self) -> AsyncRwLockWriteGuard<'_, T> {
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let inner = match timeout(DEBUG_LOCKS_DURATION_MS, self.inner.write()).await {
                    Ok(v) => v,
                    Err(_) => {
                        self.lock_id_container.report_deadlock("AsyncRwLock::write deadlock");
                    }
                };
            } else {
                let inner = self.inner.write().await;
            }
        }

        AsyncRwLockWriteGuard {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::Write,
            ),
        }
    }

    #[inline]
    #[must_use]
    pub fn try_write_arc(self: &Arc<Self>) -> Option<AsyncRwLockWriteGuardArc<T>> {
        let inner = self.inner.try_write_arc()?;

        let out = AsyncRwLockWriteGuardArc {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::TryWrite,
            ),
        };

        Some(out)
    }

    #[inline]
    pub async fn write_arc(self: &Arc<Self>) -> AsyncRwLockWriteGuardArc<T> {
        cfg_if! {
            if #[cfg(feature = "debug-locks")] {
                let inner = match timeout(DEBUG_LOCKS_DURATION_MS, self.inner.write_arc()).await {
                    Ok(v) => v,
                    Err(_) => {
                        self.lock_id_container.report_deadlock("AsyncRwLock::write_arc deadlock");
                    }
                };
            } else {
                let inner = self.inner.write_arc().await;
            }
        }

        AsyncRwLockWriteGuardArc {
            inner,
            #[cfg(feature = "debug-locks")]
            _guard_id_container: GuardIdContainer::next(
                self.lock_id_container.clone(),
                #[cfg(feature = "debug-locks-detect")]
                LockSense::Write,
            ),
        }
    }
}

impl<T> From<T> for AsyncRwLock<T> {
    #[inline]
    fn from(val: T) -> AsyncRwLock<T> {
        AsyncRwLock::new(val)
    }
}

#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct AsyncRwLockReadGuard<'a, T: ?Sized> {
    inner: InnerAsyncRwLockReadGuard<'a, T>,
    #[cfg(feature = "debug-locks")]
    _guard_id_container: GuardIdContainer,
}

impl<T: fmt::Display + ?Sized> fmt::Display for AsyncRwLockReadGuard<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: ?Sized> std::ops::Deref for AsyncRwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner
    }
}

#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct AsyncRwLockReadGuardArc<T> {
    inner: InnerAsyncRwLockReadGuardArc<T>,
    #[cfg(feature = "debug-locks")]
    _guard_id_container: GuardIdContainer,
}

impl<T: fmt::Display> fmt::Display for AsyncRwLockReadGuardArc<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> std::ops::Deref for AsyncRwLockReadGuardArc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner
    }
}

#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct AsyncRwLockWriteGuard<'a, T: ?Sized> {
    inner: InnerAsyncRwLockWriteGuard<'a, T>,
    #[cfg(feature = "debug-locks")]
    _guard_id_container: GuardIdContainer,
}

impl<T: fmt::Display + ?Sized> fmt::Display for AsyncRwLockWriteGuard<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: ?Sized> std::ops::Deref for AsyncRwLockWriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: ?Sized> std::ops::DerefMut for AsyncRwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct AsyncRwLockWriteGuardArc<T: ?Sized> {
    inner: InnerAsyncRwLockWriteGuardArc<T>,
    #[cfg(feature = "debug-locks")]
    _guard_id_container: GuardIdContainer,
}

impl<T: fmt::Display + ?Sized> fmt::Display for AsyncRwLockWriteGuardArc<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T: ?Sized> std::ops::Deref for AsyncRwLockWriteGuardArc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: ?Sized> std::ops::DerefMut for AsyncRwLockWriteGuardArc<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
