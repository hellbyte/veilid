//! Test suite utilities for non-wasm platforms
#![cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::*;

///////////////////////////////////////////////////////////////////////////

cfg_if::cfg_if! {
    if #[cfg(feature = "rt-async-std")] {
        #[allow(dead_code)]
        pub fn block_on<F: Future<Output = T>, T>(f: F) -> T {
            async_std::task::block_on(f)
        }
    } else if #[cfg(feature = "rt-tokio")] {
        #[allow(dead_code)]
        pub fn block_on<F: Future<Output = T>, T>(f: F) -> T {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(f)
        }
    } else {
        compile_error!("needs executor implementation");
    }
}

///////////////////////////////////////////////////////////////////////////
cfg_if! {
    if #[cfg(test)] {
        use serial_test::serial;
        use std::sync::Once;
        use paste::paste;

        macro_rules! run_test {
            // Nearly all test runner code is cookie cutter, and copy-pasting makes it too easy to make a typo.

            // Pass in a module and test module, and we'll run its `test_all`.
            (module $parent_module:ident $test_module:ident) => {
                paste! {
                    #[test]
                    #[serial]
                    fn [<run_ $parent_module _ $test_module>]() {
                        setup();
                        block_on(async {
                            $parent_module::tests::$test_module::test_all().await;
                        })
                    }
                }
            };

            // Pass in a component name, and we'll run its `tests::test_all`.
            (component $component:ident) => {
                paste! {
                    #[test]
                    #[serial]
                    fn [<run_tests_ $component>]() {
                        setup();
                        block_on(async {
                            $component::tests::test_all().await;
                        })
                    }
                }
            };

            // Pass in a 'common' test module name, and we'll run its `test_all`.
            (common $test_module:ident) => {
                paste! {
                    #[test]
                    #[serial]
                    fn [<run_ $test_module>]() {
                        setup();
                        block_on(async {
                            tests::$test_module::test_all().await;
                        })
                    }
                }
            };

        }

        static SETUP_ONCE: Once = Once::new();

        pub fn setup() {
            SETUP_ONCE.call_once(|| {
                use tracing_subscriber::{EnvFilter, fmt, prelude::*};
                let mut env_filter = EnvFilter::builder().from_env_lossy();
                for ig in DEFAULT_LOG_FACILITIES_IGNORE_LIST {
                    env_filter = env_filter.add_directive(format!("{}=off", ig).parse().unwrap());
                }
                let fmt_layer = fmt::layer();
                tracing_subscriber::registry()
                    .with(fmt_layer)
                    .with(env_filter)
                    .init();
            });
        }

        run_test!(module crypto test_types);
        run_test!(module crypto test_crypto);
        run_test!(module crypto test_envelope_receipt);

        run_test!(common test_veilid_core);
        run_test!(common test_protected_store);

        run_test!(module table_store test_table_store);

        run_test!(component veilid_api);

        run_test!(component routing_table);

        run_test!(component network_manager);

        run_test!(component storage_manager);

    }
}
