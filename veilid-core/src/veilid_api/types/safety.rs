use super::*;

// Ordering here matters, >= is used to check strength of sequencing requirement
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    GetSize,
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi, namespace)
)]
#[must_use]
#[derive(Default)]
pub enum Sequencing {
    #[default]
    NoPreference = 0,
    PreferOrdered = 1,
    EnsureOrdered = 2,
}

impl Sequencing {
    #[must_use]
    pub fn matches_ordering(&self, ordering: SequenceOrdering) -> bool {
        match self {
            Sequencing::NoPreference => true,
            Sequencing::PreferOrdered => true,
            Sequencing::EnsureOrdered => match ordering {
                SequenceOrdering::Unordered => false,
                SequenceOrdering::Ordered => true,
            },
        }
    }
}

impl fmt::Display for Sequencing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Sequencing::NoPreference => "nop",
            Sequencing::PreferOrdered => "pre",
            Sequencing::EnsureOrdered => "ens",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for Sequencing {
    type Err = VeilidAPIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        if "nopreference".starts_with(&s) || s.is_empty() {
            Ok(Self::NoPreference)
        } else if "preferordered".starts_with(&s) || s == "ord" {
            Ok(Self::PreferOrdered)
        } else if "ensureordered".starts_with(&s) || s == "*ord" {
            Ok(Self::EnsureOrdered)
        } else {
            Err(VeilidAPIError::parse_error("invalid sequencing str", s))
        }
    }
}

// Ordering here matters, >= is used to check strength of sequencing requirement
#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, PartialOrd, Ord, EnumSetType, Hash, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi, namespace)
)]
#[must_use]
#[enumset(repr = "u8")]
pub enum SequenceOrdering {
    Unordered = 0,
    Ordered = 1,
}

impl SequenceOrdering {
    /// The sequencing requirement that guarantees this ordering
    pub fn strict_sequencing(&self) -> Sequencing {
        match self {
            SequenceOrdering::Unordered => Sequencing::NoPreference,
            SequenceOrdering::Ordered => Sequencing::EnsureOrdered,
        }
    }
    /// The lowest sequencing requirement that matches this ordering
    pub fn minimum_sequencing(&self) -> Sequencing {
        match self {
            SequenceOrdering::Unordered => Sequencing::NoPreference,
            SequenceOrdering::Ordered => Sequencing::PreferOrdered,
        }
    }
    /// The highest sequencing requirement that allows this ordering
    pub fn maximum_sequencing(&self) -> Sequencing {
        match self {
            SequenceOrdering::Unordered => Sequencing::PreferOrdered,
            SequenceOrdering::Ordered => Sequencing::EnsureOrdered,
        }
    }
}

pub type SequenceOrderingSet = EnumSet<SequenceOrdering>;

// Ordering here matters, >= is used to check strength of stability requirement
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    GetSize,
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi, namespace)
)]
#[must_use]
#[derive(Default)]
pub enum Stability {
    #[default]
    LowLatency = 0,
    Reliable = 1,
}

impl fmt::Display for Stability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Stability::LowLatency => "low",
            Stability::Reliable => "rel",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for Stability {
    type Err = VeilidAPIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        if "lowlatency".starts_with(&s) {
            Ok(Self::LowLatency)
        } else if "reliable".starts_with(&s) {
            Ok(Self::Reliable)
        } else {
            Err(VeilidAPIError::parse_error("invalid stability str", s))
        }
    }
}

/// The choice of safety route to include in compiled routes.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema, GetSize,
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi, namespace)
)]
#[must_use]
pub enum SafetySelection {
    /// Don't use a safety route, only specify the sequencing preference.
    Unsafe(Sequencing),
    /// Use a safety route and parameters specified by a SafetySpec.
    Safe(SafetySpec),
}

impl SafetySelection {
    pub fn get_sequencing(&self) -> Sequencing {
        match self {
            SafetySelection::Unsafe(seq) => *seq,
            SafetySelection::Safe(ss) => ss.sequencing,
        }
    }
}

impl Default for SafetySelection {
    fn default() -> Self {
        Self::Unsafe(Sequencing::NoPreference)
    }
}

/// Options for safety routes (sender privacy).
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema, GetSize,
)]
#[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(Tsify))]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct SafetySpec {
    /// Preferred safety route set id if it still exists.
    #[schemars(with = "Option<String>")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        tsify(optional, type = "string")
    )]
    pub preferred_route: Option<RouteId>,
    /// Must be greater than 0.
    pub hop_count: usize,
    /// Prefer reliability over speed.
    pub stability: Stability,
    /// Prefer connection-oriented sequenced protocols.
    pub sequencing: Sequencing,
}
