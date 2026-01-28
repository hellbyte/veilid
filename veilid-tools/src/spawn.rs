use super::*;

cfg_if! {
    if #[cfg(feature="rt-wasm-bindgen")] {
        use async_executors::{Bindgen, LocalSpawnHandleExt, SpawnHandleExt};

        cfg_if! {
            if #[cfg(feature="debug-locks-detect")] {
                use std::task::{Context, Poll, Wake, Waker};
                use std::sync::{atomic::AtomicU64, LazyLock};
                use send_wrapper::SendWrapper;

                static ACTIVE_TASK_ID: LazyLock<SendWrapper<AtomicU64>> = LazyLock::new(|| SendWrapper::new(AtomicU64::new(0)));
                static NEXT_TASK_ID: LazyLock<SendWrapper<AtomicU64>> = LazyLock::new(|| SendWrapper::new(AtomicU64::new(0)));

                #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
                pub struct AsyncTaskId(u64);
                impl AsyncTaskId {
                    #[must_use]
                    pub fn this() -> AsyncTaskId {
                        AsyncTaskId(ACTIVE_TASK_ID.load(Ordering::Relaxed))
                    }
                }

                // Wrapper for waker that propagates a task id
                struct AllocTaskIdWakerWrapper {
                    inner_waker: Waker,
                    task_id: u64,
                }
                impl<'a> Wake for AllocTaskIdWakerWrapper {
                    fn wake(self: Arc<Self>) {
                        ACTIVE_TASK_ID.store(self.task_id, Ordering::Relaxed);
                        self.inner_waker.wake_by_ref();
                    }
                }

                // Wrapper that adds a task id to the context of a future that is the start of a spawned task
                struct AllocTaskIdFuture<Fut: Future> {
                    inner: Fut,
                    task_id: u64,
                }
                impl<Fut: Future> From<Fut> for AllocTaskIdFuture<Fut> {
                    fn from(inner: Fut) -> Self {
                        let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
                        Self {
                            inner,
                            task_id,
                        }
                    }
                }
                impl<Fut: Future> Future for AllocTaskIdFuture<Fut> {
                    type Output = Fut::Output;
                    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                        // Poll inner future
                        let task_id = self.task_id;
                        let inner_fut = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
                        let wrapped_waker = Arc::new(AllocTaskIdWakerWrapper {
                            inner_waker: cx.waker().clone(),
                            task_id,
                        }).into();
                        let mut wrapped_cx = Context::from_waker(&wrapped_waker);
                        inner_fut.poll(&mut wrapped_cx)
                    }
                }
            }
        }

        pub fn spawn<Out>(_name: &str, future: impl Future<Output = Out> + Send + 'static) -> MustJoinHandle<Out>
        where
            Out: Send + 'static,
        {

            #[cfg(feature="debug-locks-detect")]
            let future = AllocTaskIdFuture::from(future);

            MustJoinHandle::new(
                Bindgen
                    .spawn_handle(future)
                    .expect_or_log("wasm-bindgen-futures spawn_handle_local should never error out"),
            )
        }

        pub fn spawn_local<Out>(_name: &str, future: impl Future<Output = Out> + 'static) -> MustJoinHandle<Out>
        where
            Out: 'static,
        {

            #[cfg(feature="debug-locks-detect")]
            let future = AllocTaskIdFuture::from(future);

            MustJoinHandle::new(
                Bindgen
                    .spawn_handle_local(future)
                    .expect_or_log("wasm-bindgen-futures spawn_handle_local should never error out"),
            )
        }

        pub fn spawn_detached<Out>(_name: &str, future: impl Future<Output = Out> + Send + 'static)
        where
            Out: Send + 'static,
        {
            #[cfg(feature="debug-locks-detect")]
            let future = AllocTaskIdFuture::from(future);

            Bindgen
                .spawn_handle_local(future)
                .expect_or_log("wasm-bindgen-futures spawn_handle_local should never error out")
                .detach()
        }
        pub fn spawn_detached_local<Out>(_name: &str, future: impl Future<Output = Out> + 'static)
        where
            Out: 'static,
        {
            #[cfg(feature="debug-locks-detect")]
            let future = AllocTaskIdFuture::from(future);

            Bindgen
                .spawn_handle_local(future)
                .expect_or_log("wasm-bindgen-futures spawn_handle_local should never error out")
                .detach()
        }

    } else {

        pub fn spawn<Out>(name: &str, future: impl Future<Output = Out> + Send + 'static) -> MustJoinHandle<Out>
        where
            Out: Send + 'static,
        {
            cfg_if! {
                if #[cfg(feature="rt-async-std")] {
                    MustJoinHandle::new(async_std::task::Builder::new().name(name.to_string()).spawn(future).unwrap_or_log())
                } else if #[cfg(all(tokio_unstable, feature="rt-tokio", feature="tracing"))] {
                    MustJoinHandle::new(tokio::task::Builder::new().name(name).spawn(future).unwrap_or_log())
                } else if #[cfg(feature="rt-tokio")] {
                    let _name = name;
                    MustJoinHandle::new(tokio::task::spawn(future))
                }
            }
        }

        pub fn spawn_local<Out>(name: &str, future: impl Future<Output = Out> + 'static) -> MustJoinHandle<Out>
        where
            Out: 'static,
        {
            cfg_if! {
                if #[cfg(feature="rt-async-std")] {
                    MustJoinHandle::new(async_std::task::Builder::new().name(name.to_string()).local(future).unwrap_or_log())
                } else if #[cfg(all(tokio_unstable, feature="rt-tokio", feature="tracing"))] {
                    MustJoinHandle::new(tokio::task::Builder::new().name(name).spawn_local(future).unwrap_or_log())
                } else if #[cfg(feature="rt-tokio")] {
                    let _name = name;
                    MustJoinHandle::new(tokio::task::spawn_local(future))
                }
            }
        }

        pub fn spawn_detached<Out>(name: &str, future: impl Future<Output = Out> + Send + 'static)
        where
            Out: Send + 'static,
        {
            cfg_if! {
                if #[cfg(feature="rt-async-std")] {
                    drop(async_std::task::Builder::new().name(name.to_string()).spawn(future).unwrap_or_log());
                } else if #[cfg(all(tokio_unstable, feature="rt-tokio", feature="tracing"))] {
                    drop(tokio::task::Builder::new().name(name).spawn(future).unwrap_or_log());
                } else if #[cfg(feature="rt-tokio")] {
                    let _name = name;
                    drop(tokio::task::spawn(future))
                }
            }
        }

        pub fn spawn_detached_local<Out>(name: &str,future: impl Future<Output = Out> + 'static)
        where
            Out: 'static,
        {
            cfg_if! {
                if #[cfg(feature="rt-async-std")] {
                    drop(async_std::task::Builder::new().name(name.to_string()).local(future).unwrap_or_log());
                } else if #[cfg(all(tokio_unstable, feature="rt-tokio", feature="tracing"))] {
                    drop(tokio::task::Builder::new().name(name).spawn_local(future).unwrap_or_log());
                } else if #[cfg(feature="rt-tokio")] {
                    let _name = name;
                    drop(tokio::task::spawn_local(future))
                }
            }
        }

        #[allow(unused_variables)]
        pub async fn blocking_wrapper<F, R>(name: &str, blocking_task: F, err_result: R) -> R
        where
            F: FnOnce() -> R + Send + 'static,
            R: Send + 'static,
        {
            // run blocking stuff in blocking thread
            cfg_if! {
                if #[cfg(feature="rt-async-std")] {
                    let _name = name;
                    // async_std::task::Builder blocking doesn't work like spawn_blocking()
                    async_std::task::spawn_blocking(blocking_task).await
                } else if #[cfg(all(tokio_unstable, feature="rt-tokio", feature="tracing"))] {
                    tokio::task::Builder::new().name(name).spawn_blocking(blocking_task).unwrap_or_log().await.unwrap_or(err_result)
                } else if #[cfg(feature="rt-tokio")] {
                    let _name = name;
                    tokio::task::spawn_blocking(blocking_task).await.unwrap_or(err_result)
                } else {
                    #[compile_error("must use an executor")]
                }
            }
        }

        cfg_if! {
            if #[cfg(feature="rt-tokio")] {
                #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
                pub struct AsyncTaskId(tokio::task::Id);
                impl AsyncTaskId {
                    #[must_use]
                    pub fn this() -> AsyncTaskId {
                        AsyncTaskId(tokio::task::id())
                    }
                }
            } else if #[cfg(feature="rt-async-std")] {
                #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
                pub struct AsyncTaskId(async_std::task::TaskId);
                impl AsyncTaskId {
                    #[must_use]
                    pub fn this() -> AsyncTaskId {
                        AsyncTaskId(async_std::task::current().id())
                    }
                }
            } else {
                #[compile_error("must use an executor")]
            }
        }
    }
}
