use super::*;

/// Log level for VeilidCore.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Serialize, Deserialize, JsonSchema,
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(namespace)
)]
#[must_use]
pub enum VeilidLogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl From<VeilidConfigLogLevel> for Option<VeilidLogLevel> {
    fn from(value: VeilidConfigLogLevel) -> Self {
        match value {
            VeilidConfigLogLevel::Off => None,
            VeilidConfigLogLevel::Error => Some(VeilidLogLevel::Error),
            VeilidConfigLogLevel::Warn => Some(VeilidLogLevel::Warn),
            VeilidConfigLogLevel::Info => Some(VeilidLogLevel::Info),
            VeilidConfigLogLevel::Debug => Some(VeilidLogLevel::Debug),
            VeilidConfigLogLevel::Trace => Some(VeilidLogLevel::Trace),
        }
    }
}

impl From<tracing::Level> for VeilidLogLevel {
    fn from(value: tracing::Level) -> Self {
        match value {
            tracing::Level::ERROR => VeilidLogLevel::Error,
            tracing::Level::WARN => VeilidLogLevel::Warn,
            tracing::Level::INFO => VeilidLogLevel::Info,
            tracing::Level::DEBUG => VeilidLogLevel::Debug,
            tracing::Level::TRACE => VeilidLogLevel::Trace,
        }
    }
}

impl From<VeilidLogLevel> for tracing::Level {
    fn from(val: VeilidLogLevel) -> Self {
        match val {
            VeilidLogLevel::Error => tracing::Level::ERROR,
            VeilidLogLevel::Warn => tracing::Level::WARN,
            VeilidLogLevel::Info => tracing::Level::INFO,
            VeilidLogLevel::Debug => tracing::Level::DEBUG,
            VeilidLogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl From<tracing::log::Level> for VeilidLogLevel {
    fn from(value: log::Level) -> Self {
        match value {
            tracing::log::Level::Error => VeilidLogLevel::Error,
            tracing::log::Level::Warn => VeilidLogLevel::Warn,
            tracing::log::Level::Info => VeilidLogLevel::Info,
            tracing::log::Level::Debug => VeilidLogLevel::Debug,
            tracing::log::Level::Trace => VeilidLogLevel::Trace,
        }
    }
}

impl From<VeilidLogLevel> for tracing::log::Level {
    fn from(val: VeilidLogLevel) -> Self {
        match val {
            VeilidLogLevel::Error => tracing::log::Level::Error,
            VeilidLogLevel::Warn => tracing::log::Level::Warn,
            VeilidLogLevel::Info => tracing::log::Level::Info,
            VeilidLogLevel::Debug => tracing::log::Level::Debug,
            VeilidLogLevel::Trace => tracing::log::Level::Trace,
        }
    }
}

impl TryFrom<&str> for VeilidLogLevel {
    type Error = VeilidAPIError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for VeilidLogLevel {
    type Error = VeilidAPIError;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        Self::from_str(value.as_str())
    }
}

impl TryFrom<&String> for VeilidLogLevel {
    type Error = VeilidAPIError;

    fn try_from(value: &String) -> Result<Self, <Self as TryFrom<&String>>::Error> {
        Self::from_str(value.as_str())
    }
}

impl FromStr for VeilidLogLevel {
    type Err = VeilidAPIError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "error" => Self::Error,
            "warn" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => {
                apibail_invalid_argument!("invalid VeilidLogLevel string", "s", s);
            }
        })
    }
}
impl fmt::Display for VeilidLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match self {
            Self::Error => "Error",
            Self::Warn => "Warn",
            Self::Info => "Info",
            Self::Debug => "Debug",
            Self::Trace => "Trace",
        };
        write!(f, "{}", text)
    }
}
/// A VeilidCore log message with optional backtrace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(Tsify))]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
pub struct VeilidLog {
    pub log_level: VeilidLogLevel,
    pub message: String,
    #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), tsify(optional))]
    pub backtrace: Option<String>,
}
