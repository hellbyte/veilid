use super::*;
use core::sync::atomic::{AtomicI8, AtomicU32, Ordering};
use js_sys::{global, Reflect};
use std::io;
use ws_stream_wasm::WsErr;

pub fn is_browser() -> bool {
    static CACHE: AtomicI8 = AtomicI8::new(-1);
    let cache = CACHE.load(Ordering::Acquire);
    if cache != -1 {
        return cache != 0;
    }

    let res = Reflect::has(global().as_ref(), &"navigator".into()).unwrap_or_default();

    CACHE.store(res as i8, Ordering::Release);

    res
}

pub fn is_browser_https() -> bool {
    static CACHE: AtomicI8 = AtomicI8::new(-1);
    let cache = CACHE.load(Ordering::Acquire);
    if cache != -1 {
        return cache != 0;
    }

    let res = js_sys::eval("self.location.protocol === 'https'")
        .map(|res| res.is_truthy())
        .unwrap_or_default();

    CACHE.store(res as i8, Ordering::Release);

    res
}

static IPV6_IS_SUPPORTED: Mutex<Option<bool>> = Mutex::new(None);

pub fn is_ipv6_supported() -> bool {
    let mut opt_supp = IPV6_IS_SUPPORTED.lock();
    if let Some(supp) = *opt_supp {
        return supp;
    }

    // XXX: See issue #92
    let supp = false;

    *opt_supp = Some(supp);
    supp
}

pub fn get_concurrency() -> u32 {
    static CACHE: AtomicU32 = AtomicU32::new(0);
    let cache = CACHE.load(Ordering::Acquire);
    if cache != 0 {
        return cache;
    }

    let res = js_sys::eval("navigator.hardwareConcurrency")
        .map(|res| res.as_f64().unwrap_or(1.0f64) as u32)
        .unwrap_or(1);

    CACHE.store(res, Ordering::Release);

    res
}

#[must_use]
pub fn ws_err_to_io_error(err: WsErr) -> io::Error {
    match err {
        WsErr::InvalidWsState { supplied: _ } => {
            io::Error::new(io::ErrorKind::InvalidInput, err.to_string())
        }
        WsErr::ConnectionNotOpen => io::Error::new(io::ErrorKind::NotConnected, err.to_string()),
        WsErr::InvalidUrl { supplied: _ } => {
            io::Error::new(io::ErrorKind::InvalidInput, err.to_string())
        }
        WsErr::InvalidCloseCode { supplied: _ } => {
            io::Error::new(io::ErrorKind::InvalidInput, err.to_string())
        }
        WsErr::ReasonStringToLong => io::Error::new(io::ErrorKind::InvalidInput, err.to_string()),
        WsErr::ConnectionFailed { event: _ } => {
            io::Error::new(io::ErrorKind::ConnectionRefused, err.to_string())
        }
        WsErr::InvalidEncoding => io::Error::new(io::ErrorKind::InvalidInput, err.to_string()),
        WsErr::CantDecodeBlob => io::Error::new(io::ErrorKind::InvalidInput, err.to_string()),
        WsErr::UnknownDataType => io::Error::new(io::ErrorKind::InvalidInput, err.to_string()),
        _ => io::Error::other(err.to_string()),
    }
}
