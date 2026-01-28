#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#![no_std]
#![recursion_limit = "256"]

/// Veilid WASM Bindings
extern crate alloc;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::*;
use core::cell::RefCell;
use core::fmt::Debug;
use core::sync::atomic::{AtomicBool, Ordering};
use js_sys::*;
use lazy_static::*;
use send_wrapper::*;
use serde::*;
use tracing_subscriber::prelude::*;
use tracing_subscriber::*;
use tsify::*;
use veilid_core::*;
use veilid_core::{tools::*, VeilidAPIError};
use veilid_tracing_wasm::*;

cfg_if::cfg_if! {
    if #[cfg(feature="dart")] {
        use wasm_bindgen_futures::*;
        use futures_util::FutureExt;
        use gloo_utils::format::JsValueSerdeExt;

        pub mod dart;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="js")] {
        pub mod js;
    }
}

pub mod veilid_version;
pub mod veilid_wasm_config;

mod wasm_helpers;

pub use veilid_version::*;
pub use veilid_wasm_config::*;

use wasm_helpers::*;
