use super::*;
use num_traits::{PrimInt, Unsigned};

impl_veilid_log_facility!("stor");

#[derive(ThisError, Debug, Clone, Eq, PartialEq)]
pub enum ConcurrencyError {
    #[error("ConcurrentAccess({description})")]
    ConcurrentAccess { description: String },
}
impl From<ConcurrencyError> for VeilidAPIError {
    fn from(value: ConcurrencyError) -> Self {
        Self::internal(value.to_string())
    }
}

#[derive(ThisError, Debug, Clone, Copy, Eq, PartialEq)]
pub enum LimitError<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    #[error("limit overflow")]
    OverLimit { value: T, limit: T },
}
impl<T> From<LimitError<T>> for VeilidAPIError
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn from(value: LimitError<T>) -> Self {
        Self::internal(value.to_string())
    }
}

#[derive(ThisError, Debug, Clone, Copy, Eq, PartialEq)]
pub enum NumericError<T: PrimInt + Unsigned + fmt::Display + fmt::Debug> {
    #[error("numeric overflow: current={current} added={added}")]
    Overflow { current: T, added: T },
    #[error("numeric underflow: current={current} removed={removed}")]
    Underflow { current: T, removed: T },
}
impl<T> From<NumericError<T>> for VeilidAPIError
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn from(value: NumericError<T>) -> Self {
        Self::internal(value.to_string())
    }
}

struct LimitedSizeUnlockedInner<T: PrimInt + Unsigned + fmt::Display + fmt::Debug> {
    registry: VeilidComponentRegistry,
    description: String,
    limit: Option<T>,
}

#[derive(Debug)]
struct LimitedSizeInner<T: PrimInt + Unsigned + fmt::Display + fmt::Debug> {
    value: T,
}

#[derive(Clone)]
pub struct LimitedSize<T: PrimInt + Unsigned + fmt::Display + fmt::Debug> {
    unlocked_inner: Arc<LimitedSizeUnlockedInner<T>>,
    inner: Arc<Mutex<LimitedSizeInner<T>>>,
}

impl<T> VeilidComponentRegistryAccessor for LimitedSize<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn registry(&self) -> VeilidComponentRegistry {
        self.unlocked_inner.registry.clone()
    }
}

impl<T> fmt::Debug for LimitedSize<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LimitedSize")
            .field("description", &self.unlocked_inner.description)
            .field("limit", &self.unlocked_inner.limit)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> fmt::Display for LimitedSize<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We use an unsafe read of this mutex because we don't want to
        // ever deadlock while printing the value
        write!(f, "{}", unsafe { (*self.inner.data_ptr()).value })
    }
}

impl<T: PrimInt + Unsigned + fmt::Display + fmt::Debug> LimitedSize<T> {
    pub fn new(
        registry: VeilidComponentRegistry,
        description: &str,
        limit: Option<T>,
        value: T,
    ) -> Self {
        Self {
            unlocked_inner: Arc::new(LimitedSizeUnlockedInner {
                registry,
                description: description.to_owned(),
                limit,
            }),
            inner: Arc::new(Mutex::new(LimitedSizeInner { value })),
        }
    }

    pub fn limit(&self) -> Option<T> {
        self.unlocked_inner.limit
    }

    pub fn with_value<R, F: FnOnce(T) -> R>(&self, closure: F) -> Result<R, ConcurrencyError> {
        let Some(inner) = self.inner.try_lock_arc() else {
            return Err(ConcurrencyError::ConcurrentAccess {
                description: format!(
                    "Concurrent attempt to modify LimitedSize({})",
                    self.unlocked_inner.description
                ),
            });
        };
        Ok(closure(inner.value))
    }

    pub fn modify(&self) -> Result<LimitedSizeGuard<T>, ConcurrencyError> {
        let Some(inner) = self.inner.try_lock_arc() else {
            return Err(ConcurrencyError::ConcurrentAccess {
                description: format!(
                    "Concurrent attempt to modify LimitedSize({})",
                    self.unlocked_inner.description
                ),
            });
        };

        Ok(LimitedSizeGuard::new(self.unlocked_inner.clone(), inner))
    }
}

pub struct LimitedSizeGuard<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    unlocked_inner: Arc<LimitedSizeUnlockedInner<T>>,
    inner: ArcMutexGuard<RawMutex, LimitedSizeInner<T>>,
    uncommitted_value: Option<T>,
}

impl<T> VeilidComponentRegistryAccessor for LimitedSizeGuard<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn registry(&self) -> VeilidComponentRegistry {
        self.unlocked_inner.registry.clone()
    }
}

impl<T> LimitedSizeGuard<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn new(
        unlocked_inner: Arc<LimitedSizeUnlockedInner<T>>,
        inner: ArcMutexGuard<RawMutex, LimitedSizeInner<T>>,
    ) -> Self {
        Self {
            uncommitted_value: Some(inner.value),
            unlocked_inner,
            inner,
        }
    }

    pub fn set(&mut self, new_value: T) {
        self.uncommitted_value = Some(new_value);
    }

    pub fn add(&mut self, v: T) -> Result<T, NumericError<T>> {
        let uncommitted_value = self.uncommitted_value.as_mut().unwrap();
        let max_v = T::max_value() - *uncommitted_value;
        if v > max_v {
            return Err(NumericError::Overflow {
                current: *uncommitted_value,
                added: v,
            });
        }
        let new_value = *uncommitted_value + v;
        *uncommitted_value = new_value;
        Ok(new_value)
    }
    pub fn sub(&mut self, v: T) -> Result<T, NumericError<T>> {
        let uncommitted_value = self.uncommitted_value.as_mut().unwrap();
        let max_v = *uncommitted_value - T::min_value();
        if v > max_v {
            return Err(NumericError::Underflow {
                current: *uncommitted_value,
                removed: v,
            });
        }
        let new_value = *uncommitted_value - v;
        *uncommitted_value = new_value;
        Ok(new_value)
    }

    pub fn check_limit(&self) -> bool {
        if let Some(limit) = self.unlocked_inner.limit {
            let uncommitted_value = self.uncommitted_value.as_ref().unwrap();
            if *uncommitted_value > limit {
                return false;
            }
        }
        true
    }

    pub fn commit(mut self) -> Result<T, LimitError<T>> {
        let uncommitted_value = self.uncommitted_value.take().unwrap();

        if let Some(limit) = self.unlocked_inner.limit {
            if uncommitted_value > limit {
                veilid_log!(self debug "Commit over limit failed, rolled back LimitedSize({}): {} > {}", self.unlocked_inner.description, uncommitted_value, limit);
                return Err(LimitError::OverLimit {
                    value: uncommitted_value,
                    limit,
                });
            }
        }
        veilid_log!(self trace "Commit LimitedSize({}): {} => {}", self.unlocked_inner.description, self.inner.value, uncommitted_value);
        self.inner.value = uncommitted_value;
        Ok(self.inner.value)
    }

    pub fn rollback(mut self) {
        if let Some(uv) = self.uncommitted_value.take() {
            veilid_log!(self trace "Rollback LimitedSize({}): {} (drop {})", self.unlocked_inner.description, self.inner.value, uv);
        }
    }
}

impl<T> Drop for LimitedSizeGuard<T>
where
    T: PrimInt + Unsigned + fmt::Display + fmt::Debug,
{
    fn drop(&mut self) {
        if let Some(uv) = self.uncommitted_value.take() {
            veilid_log!(self trace "Drop of uncommitted LimitedSize({}): {} (drop {})", self.unlocked_inner.description, self.inner.value, uv);
        }
    }
}
