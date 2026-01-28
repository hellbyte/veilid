use super::native::*;
use super::*;

use std::backtrace::Backtrace;
use std::panic;

use jni::{objects::JClass, objects::JObject, JNIEnv};

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_veilid_veilid_1tools_1android_1tests_MainActivity_run_1tests(
    _env: JNIEnv,
    _class: JClass,
    _ctx: JObject,
) {
    veilid_tools_setup_android_tests();
    block_on(async {
        run_all_tests().await;
    })
}

pub fn veilid_tools_setup_android_tests() {
    cfg_if! {
            if #[cfg(feature = "tracing")] {
                use tracing::level_filters::LevelFilter;
                use tracing_subscriber::prelude::*;
                use tracing_subscriber::filter::Targets;

                let mut filters = Targets::default();
                filters = filters.with_default(LevelFilter::OFF);
                filters = filters.with_target("veilid_tools", LevelFilter::INFO);
                tracing_subscriber::registry()
                    .with(paranoid_android::layer("com.veilid.veilidtools-tests")
                    .with_target(true)
                    .with_filter(filters))
                    .init();
            } else {
                use log::LevelFilter;
                use android_logger::{Config,FilterBuilder};

                let mut builder = FilterBuilder::new();
                builder.filter_level(LevelFilter::Info);
    //            builder.filter_module("veilid_tools", LevelFilter::Info);
                android_logger::init_once(
                    Config::default()
                        .with_tag("veilid_tools")
                        .with_filter(builder.build())
                );
            }
        }

    // Set up panic hook for backtraces
    panic::set_hook(Box::new(|panic_info| {
        let bt = Backtrace::capture();
        if let Some(location) = panic_info.location() {
            error!(
                "panic occurred in file '{}' at line {}",
                location.file(),
                location.line(),
            );
        } else {
            error!("panic occurred but can't get location information...");
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error!("panic payload: {:?}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            error!("panic payload: {:?}", s);
        } else if let Some(a) = panic_info.payload().downcast_ref::<std::fmt::Arguments>() {
            error!("panic payload: {:?}", a);
        } else {
            error!("no panic payload");
        }
        error!("Backtrace:\n{:?}", bt);
    }));
}
