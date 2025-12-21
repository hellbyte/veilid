pub static DEFAULT_LOG_FACILITIES_IGNORE_LIST: &[&str] = &[
    "mio",
    "h2",
    "hyper",
    "tower",
    "tonic",
    "tokio",
    "runtime",
    "tokio_util",
    "want",
    "serial_test",
    "async_std",
    "async_io",
    "polling",
    "rustls",
    "async_tungstenite",
    "tungstenite",
    "netlink_proto",
    "netlink_sys",
    "hickory_resolver",
    "hickory_proto",
    "attohttpc",
    "ws_stream_wasm",
    "keyvaluedb_web",
    "veilid_api",
    "network_result",
    "dht",
    "watch",
    "fanout",
    "receipt",
    "rpc_message",
    #[cfg(feature = "geolocation")]
    "maxminddb",
];

pub static FLAME_LOG_FACILITIES_IGNORE_LIST: &[&str] = &[
    "mio",
    "h2",
    "hyper",
    "tower",
    "tonic",
    "tokio",
    "runtime",
    "tokio_util",
    "want",
    "serial_test",
    "async_std",
    "async_io",
    "polling",
    "rustls",
    "async_tungstenite",
    "tungstenite",
    "netlink_proto",
    "netlink_sys",
    "hickory_resolver",
    "hickory_proto",
    "attohttpc",
    "ws_stream_wasm",
    #[cfg(feature = "geolocation")]
    "maxminddb",
];

pub static DEFAULT_LOG_FACILITIES_ENABLED_LIST: &[&str] = &[
    "net",
    "rpc",
    "rtab",
    "stor",
    "client_api",
    "pstore",
    "tstore",
    "crypto",
    "veilid_debug",
];

#[macro_export]
macro_rules! impl_veilid_log_facility {
    ($facility:literal) => {
        const __VEILID_LOG_FACILITY: &'static str = $facility;
    };
}

#[macro_export]
macro_rules! fn_string {
    ($text:expr) => {
        || $text.to_string()
    };
}

#[macro_export]
macro_rules! log_veilid_api_error {
    ($self_expr:ident) => {
        |e: &$crate::VeilidAPIError| {
            match e.log_level() {
                $crate::Level::ERROR => {
                    veilid_log!($self_expr error "error = {}", e);
                }
                $crate::Level::WARN => {
                    veilid_log!($self_expr warn "error = {}", e);
                }
                $crate::Level::INFO => {
                    veilid_log!($self_expr info "error = {}", e);
                }
                $crate::Level::DEBUG => {
                    veilid_log!($self_expr debug "error = {}", e);
                }
                $crate::Level::TRACE => {
                    veilid_log!($self_expr trace "error = {}", e);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! veilid_log_err {
    ($self_expr:expr) => {
        |e| veilid_log_event!($self_expr, prefix: "", level: $crate::Level::ERROR, "{}", e)
    };
    ($self_expr:expr, $message:expr) => {
        |e| veilid_log_event!($self_expr, prefix: "", level: $crate::Level::ERROR, "{}: {}", $message, e)
    };
    ($self_expr:expr, $fmt:expr, $($args:tt)*) => {
        |e| veilid_log_event!($self_expr, prefix: "", level: $crate::Level::ERROR, concat!($fmt,": {}"), $($args)*, e)
    };
}

#[macro_export]
macro_rules! veilid_log_dbg {
    ($self_expr:expr) => {
        |e| veilid_log_event!($self_expr, prefix: "", level: $crate::Level::DEBUG, "{}", e)
    };
    ($self_expr:expr, $message:expr) => {
        |e| veilid_log_event!($self_expr, prefix: "", level: $crate::Level::DEBUG, "{}: {}", $message, e)
    };
    ($self_expr:expr, $fmt:expr, $($args:tt)*) => {
        |e| veilid_log_event!($self_expr, prefix: "", level: $crate::Level::DEBUG, concat!($fmt,": {}"), $($args)*, e)
    };
}

#[macro_export]
macro_rules! veilid_log_event {
    // veilid_log_event!(self, prefix:"", level: Level::XXX, "message")
    ($self_expr:expr, prefix: $prefix:literal, level: $lvl:expr, $text:expr) => {event!(
        target: self::__VEILID_LOG_FACILITY,
        $lvl,
        __VEILID_LOG_KEY = $self_expr.log_key(),
        concat!($prefix,"{}"),
        $text)
    };
    // veilid_log!(self, prefix:"", level: Level::XXX, target: "facility", "message")
    ($self_expr:expr, prefix: $prefix:literal, level: $lvl:expr, target: $target:expr, $text:expr) => {event!(
        target: $target,
        $lvl,
        __VEILID_LOG_KEY = $self_expr.log_key(),
        concat!($prefix,"{}"),
        $text)
    };
    // veilid_log!(self, prefix:"", level: Level::XXX, "data: {}", data)
    ($self_expr:expr, prefix: $prefix:literal, level: $lvl:expr, $fmt:expr, $($args:tt)*) => {event!(
        target: self::__VEILID_LOG_FACILITY,
        $lvl,
        __VEILID_LOG_KEY = $self_expr.log_key(),
        concat!($prefix,$fmt),
        $($args)*)
    };
    // veilid_log!(self, prefix:"", level: Level::XXX, target: "facility", "data: {}", data)
    ($self_expr:expr, prefix: $prefix:literal, level: $lvl:expr, target: $target:expr, $fmt:expr, $($args:tt)*) => {event!(
        target: $target,
        $lvl,
        __VEILID_LOG_KEY = $self_expr.log_key(),
        concat!($prefix,$fmt),
        $($args)*)
    };
    // veilid_log!(self, prefix:"", level: Level::XXX, fields: field=value, ?other_field)
    ($self_expr:expr, prefix: $prefix:literal, level: $lvl:expr, fields: $($k:ident).+ = $($fields:tt)*) => {event!(
        target: self::__VEILID_LOG_FACILITY,
        $lvl,
        __VEILID_LOG_KEY = $self_expr.log_key(),
        $($k).+ = $($fields)*,
        concat!($prefix,""))
    };
    // veilid_log!(self, prefix:"", Level::XXX, target: "facility", fields: field=value, ?other_field)
    ($self_expr:expr, prefix: $prefix:literal, level: $lvl:expr, target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {event!(
        target: $target,
        $lvl,
        __VEILID_LOG_KEY = $self_expr.log_key(),
        $($k).+ = $($fields)*,
        concat!($prefix,""))
    };
}

#[cfg(debug_assertions)]
pub(crate) const DEBUGWARN: tracing::Level = tracing::Level::DEBUG;
#[cfg(not(debug_assertions))]
pub(crate) const DEBUGWARN: tracing::Level = tracing::Level::WARN;

#[macro_export]
macro_rules! veilid_log {

    // ERROR //////////////////////////////////////////////////////////////////////////
    // veilid_log!(self error "message")
    ($self_ident:ident error $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::ERROR, $text)};
    // veilid_log!(self error target: "facility", "message")
    ($self_ident:ident error target: $target:expr, $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::ERROR, target: $target, $text)};
    // veilid_log!(self error "data: {}", data)
    ($self_ident:ident error $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::ERROR, $fmt, $($args)*)};
    // veilid_log!(self error target: "facility", "data: {}", data)
    ($self_ident:ident error target: $target:expr, $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::ERROR, target: $target, $fmt, $($args)*)};
    // veilid_log!(self error, fields: field=value, ?other_field)
    ($self_ident:ident error, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident level: $crate::Level::ERROR, fields: $($k).+ = $($fields)*)};
    // veilid_log!(self error target: "facility", fields: field=value, ?other_field)
    ($self_ident:ident error target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::ERROR, target: $target, fields: $($k).+ = $($fields)*)};

    // WARN //////////////////////////////////////////////////////////////////////////
    // veilid_log!(self warn "message")
    ($self_ident:ident warn $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::WARN, $text)};
    // veilid_log!(self warn target: "facility", "message")
    ($self_ident:ident warn target: $target:expr, $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::WARN, target: $target, $text)};
    // veilid_log!(self warn "data: {}", data)
    ($self_ident:ident warn $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::WARN, $fmt, $($args)*)};
    // veilid_log!(self warn target: "facility", "data: {}", data)
    ($self_ident:ident warn target: $target:expr, $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::WARN, target: $target, $fmt, $($args)*)};
    // veilid_log!(self warn, fields: field=value, ?other_field)
    ($self_ident:ident warn, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::WARN, fields: $($k).+ = $($fields)*)};
    // veilid_log!(self warn target: "facility", fields: field=value, ?other_field)
    ($self_ident:ident warn target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::WARN, target: $target, fields: $($k).+ = $($fields)*)};

    // INFO //////////////////////////////////////////////////////////////////////////
    // veilid_log!(self info "message")
    ($self_ident:ident info $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::INFO, $text)};
    // veilid_log!(self info target: "facility", "message")
    ($self_ident:ident info target: $target:expr, $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::INFO, target: $target, $text)};
    // veilid_log!(self info "data: {}", data)
    ($self_ident:ident info $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::INFO, $fmt, $($args)*)};
    // veilid_log!(self info target: "facility", "data: {}", data)
    ($self_ident:ident info target: $target:expr, $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::INFO, target: $target, $fmt, $($args)*)};
    // veilid_log!(self info, fields: field=value, ?other_field)
    ($self_ident:ident info, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::INFO, fields: $($k).+ = $($fields)*)};
    // veilid_log!(self info target: "facility", fields: field=value, ?other_field)
    ($self_ident:ident info target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::INFO, target: $target, fields: $($k).+ = $($fields)*)};

    // DEBUG //////////////////////////////////////////////////////////////////////////
    // veilid_log!(self debug "message")
    ($self_ident:ident debug $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::DEBUG, $text)};
    // veilid_log!(self debug target: "facility", "message")
    ($self_ident:ident debug target: $target:expr, $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::DEBUG, target: $target, $text)};
    // veilid_log!(self debug "data: {}", data)
    ($self_ident:ident debug $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::DEBUG, $fmt, $($args)*)};
    // veilid_log!(self debug target: "facility", "data: {}", data)
    ($self_ident:ident debug target: $target:expr, $fmt:literal, $($arg:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::DEBUG, target: $target, $fmt, $($arg)*)};
    // veilid_log!(self debug, fields: field=value, ?other_field)
    ($self_ident:ident debug, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::DEBUG, fields: $($k).+ = $($fields)*)};
    // veilid_log!(self debug target: "facility" fields: field=value, ?other_field)
    ($self_ident:ident debug target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::DEBUG, target: $target, fields: $($k).+ = $($fields)*)};

    // TRACE //////////////////////////////////////////////////////////////////////////
    // veilid_log!(self trace "message")
    ($self_ident:ident trace $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::TRACE, $text)};
    // veilid_log!(self trace target: "facility", "message")
    ($self_ident:ident trace target: $target:expr, $text:expr) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::TRACE, target: $target, $text)};
    // veilid_log!(self trace "data: {}", data)
    ($self_ident:ident trace $fmt:literal, $($arg:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::TRACE, $fmt, $($arg)*)};
    // veilid_log!(self trace target: "facility", "data: {}", data)
    ($self_ident:ident trace target: $target:expr, $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::TRACE, target: $target, $fmt, $($args)*)};
    // veilid_log!(self trace, fields: field=value, ?other_field)
    ($self_ident:ident trace, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::TRACE, fields: $($k).+ = $($fields)*)};
    // veilid_log!(self trace target: "facility", fields: field=value, ?other_field)
    ($self_ident:ident trace target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "", level: $crate::Level::TRACE, target: $target, fields: $($k).+ = $($fields)*)};

    // DEBUGWARN //////////////////////////////////////////////////////////////////////////
    // veilid_log!(self debugwarn "message")
    ($self_ident:ident debugwarn $text:expr) => {veilid_log_event!($self_ident, prefix: "DEBUGWARN: ", level: $crate::DEBUGWARN, $text)};
    // veilid_log!(self debugwarn target: "facility", "message")
    ($self_ident:ident debugwarn target: $target:expr, $text:expr) => {veilid_log_event!($self_ident, prefix: "DEBUGWARN: ", level: $crate::DEBUGWARN, target: $target, $text)};
    // veilid_log!(self debugwarn "data: {}", data)
    ($self_ident:ident debugwarn $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "DEBUGWARN: ", level: $crate::DEBUGWARN, $fmt, $($args)*)};
    // veilid_log!(self debugwarn target: "facility", "data: {}", data)
    ($self_ident:ident debugwarn target: $target:expr, $fmt:expr, $($args:tt)*) => {veilid_log_event!($self_ident, prefix: "DEBUGWARN: ", level: $crate::DEBUGWARN, target: $target, $fmt, $($args)*)};
    // veilid_log!(self debugwarn, fields: field=value, ?other_field)
    ($self_ident:ident debugwarn, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "DEBUGWARN", level: $crate::DEBUGWARN, fields: $($k).+ = $($fields)*)};
    // veilid_log!(self debugwarn target: "facility", fields: field=value, ?other_field)
    ($self_ident:ident debugwarn target: $target:expr, fields: $($k:ident).+ = $($fields:tt)*) => {veilid_log_event!($self_ident, prefix: "DEBUGWARN", level: $crate::DEBUGWARN, target: $target, fields: $($k).+ = $($fields)*)};
}

#[macro_export]
macro_rules! network_result_value_or_log {
    ($self:ident $r:expr => $f:expr) => {
        network_result_value_or_log!($self target: self::__VEILID_LOG_FACILITY, $r => [ "" ] $f )
    };
    ($self:ident $r:expr => [ $d:expr ] $f:expr) => {
        network_result_value_or_log!($self target: self::__VEILID_LOG_FACILITY, $r => [ $d ] $f )
    };
    ($self:ident target: $target:expr, $r:expr => $f:expr) => {
        network_result_value_or_log!($self target: $target, $r => [ "" ] $f )
    };
    ($self:ident target: $target:expr, $r:expr => [ $d:expr ] $f:expr) => { {
        let __extra_message = if debug_target_enabled!("network_result") {
            $d.to_string()
        } else {
            "".to_string()
        };
        match $r {
            NetworkResult::Timeout => {
                veilid_log!($self debug target: $target,
                    "{} at {}@{}:{} in {}{}",
                    "Timeout",
                    file!(),
                    line!(),
                    column!(),
                    fn_name::uninstantiated!(),
                    __extra_message
                );
                $f
            }
            NetworkResult::ServiceUnavailable(ref s) => {
                veilid_log!($self debug target: $target,
                    "{}({}) at {}@{}:{} in {}{}",
                    "ServiceUnavailable",
                    s,
                    file!(),
                    line!(),
                    column!(),
                    fn_name::uninstantiated!(),
                    __extra_message
                );
                $f
            }
            NetworkResult::NoConnection(ref e) => {
                veilid_log!($self debug target: $target,
                    "{}({}) at {}@{}:{} in {}{}",
                    "No connection",
                    e.to_string(),
                    file!(),
                    line!(),
                    column!(),
                    fn_name::uninstantiated!(),
                    __extra_message
                );
                $f
            }
            NetworkResult::AlreadyExists(ref e) => {
                veilid_log!($self debug target: $target,
                    "{}({}) at {}@{}:{} in {}{}",
                    "Already exists",
                    e.to_string(),
                    file!(),
                    line!(),
                    column!(),
                    fn_name::uninstantiated!(),
                    __extra_message
                );
                $f
            }
            NetworkResult::InvalidMessage(ref s) => {
                veilid_log!($self debug target: $target,
                    "{}({}) at {}@{}:{} in {}{}",
                    "Invalid message",
                    s,
                    file!(),
                    line!(),
                    column!(),
                    fn_name::uninstantiated!(),
                    __extra_message
                );
                $f
            }
            NetworkResult::Value(v) => v,
        }
    } };

}
