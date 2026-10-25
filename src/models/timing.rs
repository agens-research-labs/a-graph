//! The one way every provider measures `execution_time`.

use std::future::Future;
use std::time::Instant;

/// Runs `f` and returns its value with the seconds it took (request sent -> full response read).
pub(crate) async fn timed<T>(f: impl Future<Output = T>) -> (T, f64) {
    let start = Instant::now();
    let value = f.await;
    (value, start.elapsed().as_secs_f64())
}
