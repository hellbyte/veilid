//! Fork of `tracing-unwrap` crate, modified to also allow non-tracing loggers to function

use super::*;
use std::fmt;

/// Extension trait for Result types.
pub trait ResultExt<T, E> {
    fn ok_or_log(self) -> Option<T>
    where
        E: fmt::Debug;

    fn unwrap_or_log(self) -> T
    where
        E: fmt::Debug;

    fn expect_or_log(self, msg: &str) -> T
    where
        E: fmt::Debug;

    fn unwrap_err_or_log(self) -> E
    where
        T: fmt::Debug;

    fn expect_err_or_log(self, msg: &str) -> E
    where
        T: fmt::Debug;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[inline]
    #[track_caller]
    fn ok_or_log(self) -> Option<T>
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                discarded_with("called `Result::ok_or_log` on an `Err` value", &e);
                None
            }
        }
    }

    #[inline]
    #[track_caller]
    fn unwrap_or_log(self) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => failed_with("called `Result::unwrap_or_log()` on an `Err` value", &e),
        }
    }

    #[inline]
    #[track_caller]
    fn expect_or_log(self, msg: &str) -> T
    where
        E: fmt::Debug,
    {
        match self {
            Ok(t) => t,
            Err(e) => failed_with(msg, &e),
        }
    }

    #[inline]
    #[track_caller]
    fn unwrap_err_or_log(self) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Ok(t) => failed_with("called `Result::unwrap_err_or_log()` on an `Ok` value", &t),
            Err(e) => e,
        }
    }

    #[inline]
    #[track_caller]
    fn expect_err_or_log(self, msg: &str) -> E
    where
        T: fmt::Debug,
    {
        match self {
            Ok(t) => failed_with(msg, &t),
            Err(e) => e,
        }
    }
}

/// Extension trait for Option types.
pub trait OptionExt<T> {
    fn unwrap_or_log(self) -> T;

    fn expect_or_log(self, msg: &str) -> T;

    fn unwrap_none_or_log(self)
    where
        T: fmt::Debug;

    fn expect_none_or_log(self, msg: &str)
    where
        T: fmt::Debug;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    #[track_caller]
    fn unwrap_or_log(self) -> T {
        match self {
            Some(val) => val,
            None => failed("called `Option::unwrap_or_log()` on a `None` value"),
        }
    }

    #[inline]
    #[track_caller]
    fn expect_or_log(self, msg: &str) -> T {
        match self {
            Some(val) => val,
            None => failed(msg),
        }
    }

    #[inline]
    #[track_caller]
    fn unwrap_none_or_log(self)
    where
        T: fmt::Debug,
    {
        if let Some(val) = self {
            failed_with(
                "called `Option::unwrap_none_or_log()` on a `Some` value",
                &val,
            );
        }
    }

    #[inline]
    #[track_caller]
    fn expect_none_or_log(self, msg: &str)
    where
        T: fmt::Debug,
    {
        if let Some(val) = self {
            failed_with(msg, &val);
        }
    }
}

//
// Helper functions.
//

#[inline(never)]
#[cold]
#[track_caller]
fn failed(msg: &str) -> ! {
    #[cfg(not(feature = "log-location-quiet"))]
    {
        let location = std::panic::Location::caller();
        #[cfg(feature = "tracing")]
        error!(
            unwrap.filepath = location.file(),
            unwrap.lineno = location.line(),
            unwrap.columnno = location.column(),
            "{}",
            msg
        );
        #[cfg(not(feature = "tracing"))]
        error!(
            "{}:{}:{} {}",
            location.file(),
            location.line(),
            location.column(),
            msg
        );
    }

    #[cfg(feature = "log-location-quiet")]
    error!("{}", msg);

    #[cfg(feature = "panic-quiet")]
    panic!();
    #[cfg(not(feature = "panic-quiet"))]
    panic!("{}", msg)
}

#[inline(never)]
#[cold]
#[track_caller]
fn failed_with(msg: &str, value: &dyn fmt::Debug) -> ! {
    #[cfg(not(feature = "log-location-quiet"))]
    {
        let location = std::panic::Location::caller();
        #[cfg(feature = "tracing")]
        error!(
            unwrap.filepath = location.file(),
            unwrap.lineno = location.line(),
            unwrap.columnno = location.column(),
            "{}: {:?}",
            msg,
            &value
        );
        #[cfg(not(feature = "tracing"))]
        error!(
            "{}:{}:{} {}: {:?}",
            location.file(),
            location.line(),
            location.column(),
            msg,
            &value
        );
    }

    #[cfg(feature = "log-location-quiet")]
    error!("{}: {:?}", msg, &value);

    #[cfg(feature = "panic-quiet")]
    panic!();
    #[cfg(not(feature = "panic-quiet"))]
    panic!("{}: {:?}", msg, &value);
}

#[inline(never)]
#[cold]
#[track_caller]
fn discarded_with(msg: &str, value: &dyn fmt::Debug) {
    #[cfg(not(feature = "log-location-quiet"))]
    {
        let location = std::panic::Location::caller();

        warn!(
            "{}:{}:{} {}: {:?}",
            location.file(),
            location.line(),
            location.column(),
            msg,
            &value
        );
    }

    #[cfg(feature = "log-location-quiet")]
    warn!("{}: {:?}", msg, &value);
}
