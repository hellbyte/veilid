/// Microseconds since epoch
use super::*;

aligned_u64_type!(TimestampDuration);
aligned_u64_type_default_debug_impl!(TimestampDuration);

impl fmt::Display for TimestampDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", display_duration(self.as_u64()))
    }
}

impl TimestampDuration {
    pub const fn new_secs(secs: u32) -> Self {
        TimestampDuration::new(secs as u64 * 1_000_000u64)
    }
    pub const fn new_ms(ms: u64) -> Self {
        TimestampDuration::new(ms * 1_000u64)
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn since(older: Timestamp) -> Self {
        Self::new(Timestamp::now().as_u64().saturating_sub(older.as_u64()))
    }

    pub fn since_non_decreasing(older: Timestamp) -> Self {
        Self::new(
            Timestamp::now_non_decreasing()
                .as_u64()
                .saturating_sub(older.as_u64()),
        )
    }

    pub fn seconds_u32(&self) -> Result<u32, String> {
        u32::try_from(self.as_u64() / 1_000_000u64)
            .map_err(|e| format!("could not convert to seconds: {}", e))
    }

    pub fn millis_u32(&self) -> Result<u32, String> {
        u32::try_from(self.as_u64() / 1_000u64)
            .map_err(|e| format!("could not convert to milliseconds: {}", e))
    }

    #[must_use]
    pub fn seconds_f64(&self) -> f64 {
        // Downshift precision until it fits, lose least significant bits
        let mut mul: f64 = 1.0f64 / 1_000_000.0f64;
        let mut usec = self.0;
        while usec > (u32::MAX as u64) {
            usec >>= 1;
            mul *= 2.0f64;
        }
        f64::from(usec as u32) * mul
    }

    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self::new(self.0.saturating_add(rhs.0))
    }

    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self::new(self.0.saturating_sub(rhs.0))
    }

    pub const fn saturating_mul(self, rhs: u64) -> Self {
        Self::new(self.0.saturating_mul(rhs))
    }

    pub const fn div(self, rhs: u64) -> Self {
        Self::new(self.0 / rhs)
    }

    pub const fn div_assign(&mut self, rhs: u64) {
        *self = self.div(rhs)
    }

    pub fn checked_div<T: Into<u64>>(self, rhs: T) -> Option<Self> {
        self.0.checked_div(rhs.into()).map(Self::new)
    }

    pub fn checked_mul<T: Into<u64>>(self, rhs: T) -> Option<Self> {
        self.0.checked_mul(rhs.into()).map(Self::new)
    }

    pub const fn saturating_add_assign(&mut self, rhs: Self) {
        *self = self.saturating_add(rhs);
    }
    pub const fn saturating_sub_assign(&mut self, rhs: Self) {
        *self = self.saturating_sub(rhs);
    }
    pub const fn saturating_mul_assign(&mut self, rhs: u64) {
        *self = self.saturating_mul(rhs);
    }
}
