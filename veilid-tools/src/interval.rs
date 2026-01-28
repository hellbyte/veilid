use super::*;

cfg_if! {
    if #[cfg(all(target_arch = "wasm32", target_os = "unknown"))] {

        pub fn interval<F, FUT>(name: &str, freq_ms: u32, immediate: bool, callback: F) -> PinBoxFutureStatic<()>
        where
            F: Fn() -> FUT + Send + Sync + 'static,
            FUT: Future<Output = ()> + Send,
        {
            let e = Eventual::new();

            let ie = e.clone();
            let jh = spawn(name, Box::pin(async move {
                let freq_u64 = (freq_ms as u64) * 1000u64;
                let start_tick_ts = get_raw_timestamp();

                let mut end_tick_ts = if immediate {
                    callback().await;
                    get_raw_timestamp()
                } else {
                    start_tick_ts
                };
                loop {
                    let wait_ms = ((freq_u64 - end_tick_ts.saturating_sub(start_tick_ts) % freq_u64) / 1000) as u32;
                    if timeout(wait_ms, ie.instance_clone(())).await.is_ok() {
                        break;
                    }

                    callback().await;

                    end_tick_ts = get_raw_timestamp();
                }
            }));

            Box::pin(async move {
                e.resolve().await;
                jh.await;
            })
        }

    } else {

        pub fn interval<F, FUT>(name: &str, freq_ms: u32, immediate: bool, callback: F) -> PinBoxFutureStatic<()>
        where
            F: Fn() -> FUT + Send + Sync + 'static,
            FUT: Future<Output = ()> + Send,
        {
            let e = Eventual::new();

            let ie = e.clone();
            let jh = spawn(name, async move {
                let freq_u64 = (freq_ms as u64) * 1000u64;
                let start_tick_ts = get_raw_timestamp();

                let mut end_tick_ts = if immediate {
                    callback().await;
                    get_raw_timestamp()
                } else {
                    start_tick_ts
                };
                loop {
                    let wait_ms = ((freq_u64 - end_tick_ts.saturating_sub(start_tick_ts) % freq_u64) / 1000) as u32;
                    if timeout(wait_ms, ie.instance_clone(())).await.is_ok() {
                        break;
                    }

                    callback().await;

                    end_tick_ts = get_raw_timestamp();
                }
            });

            Box::pin(async move {
                e.resolve().await;
                jh.await;
            })
        }

    }
}
