use super::*;

/////////////////////////////////////////////////////////////////////////////////
// Resolver
//
// Uses system resolver on windows and hickory-resolver elsewhere
// hickory-resolver hangs for a long time on Windows building some cache or something
// and we really should be using the built-in system resolver when possible

cfg_if! {

    /////////////////////////////////////////////////////////////////////////////
    // WASM (unsupported)

    if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {
        mod wasm;
        pub use wasm::*;
    }

    /////////////////////////////////////////////////////////////////////////////
    // Non-Windows

    else if #[cfg(not(target_os = "windows"))] {
        mod non_windows;
        pub use non_windows::*;
    }

    /////////////////////////////////////////////////////////////////////////////
    // Windows

    else {
        mod windows;
        pub use windows::*;
    }

}

#[derive(Debug, Clone, Eq, PartialEq, ThisError)]
pub enum ResolverError {
    #[error("Generic resolver error")]
    Generic(String),
}
