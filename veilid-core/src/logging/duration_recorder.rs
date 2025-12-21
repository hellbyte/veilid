use futures_util::future::{select, Either};

use super::*;

pub async fn record_duration_fut<F, R>(fut: F) -> R
where
    F: Future<Output = R>,
{
    let start = Timestamp::now_non_decreasing();
    let out = Box::pin(fut).await;
    let duration = TimestampDuration::since(start);
    tracing::Span::current().record("duration", duration.to_string());
    out
}

pub fn record_duration<C, R>(closure: C) -> R
where
    C: FnOnce() -> R,
{
    let start = Timestamp::now_non_decreasing();
    let out = closure();
    let duration = TimestampDuration::since(start);
    tracing::Span::current().record("duration", duration.to_string());
    out
}

#[expect(dead_code)]
pub fn debug_duration<C, R, D>(closure: C, limit: TimestampDuration, callback: D) -> R
where
    C: FnOnce() -> R,
    D: FnOnce(String),
{
    let start = Timestamp::now_non_decreasing();
    let out = closure();
    let duration = TimestampDuration::since(start);
    if duration > limit {
        let msg = format!("Excessive duration: {}", duration);
        callback(msg);
    }

    out
}

#[allow(dead_code)]
pub trait MeasureFuture<T, C>
where
    C: FnOnce(TimestampDuration),
{
    fn measure(self, callback: C) -> impl Future<Output = T>;
    fn measure_limit(self, limit: TimestampDuration, callback: C) -> impl Future<Output = T>;
}

#[allow(dead_code)]
pub trait MeasureDebugFuture<T, D>
where
    D: FnOnce(String),
{
    fn measure_debug(self, limit: TimestampDuration, callback: D) -> impl Future<Output = T>;
}

impl<T, C, M> MeasureFuture<T, C> for M
where
    C: FnOnce(TimestampDuration),
    M: Future<Output = T>,
{
    async fn measure(self, callback: C) -> T {
        let start = Timestamp::now_non_decreasing();
        let out = Box::pin(self).await;
        let duration = TimestampDuration::since(start);
        callback(duration);
        out
    }

    async fn measure_limit(self, limit: TimestampDuration, callback: C) -> T {
        let start = Timestamp::now_non_decreasing();
        let out = Box::pin(self).await;
        let duration = TimestampDuration::since(start);
        if duration > limit {
            callback(duration);
        }
        out
    }
}

impl<T, D, M> MeasureDebugFuture<T, D> for M
where
    D: Fn(String),
    M: Future<Output = T>,
{
    async fn measure_debug(self, limit: TimestampDuration, callback: D) -> T {
        let start = Timestamp::now_non_decreasing();

        let res = select(Box::pin(self), Box::pin(sleep(limit.millis_u32().unwrap()))).await;
        let out = match res {
            Either::Left((out, sleep_fut)) => {
                drop(sleep_fut);
                out
            }
            Either::Right((_, fut)) => {
                let msg = format!("Duration limit exceeded: {}", limit);
                callback(msg);
                fut.await
            }
        };
        let duration = TimestampDuration::since(start);
        if duration > limit {
            let msg = format!("Excessive duration: {}", duration);
            callback(msg);
        }
        out
    }
}
