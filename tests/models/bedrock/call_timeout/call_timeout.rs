use super::*;
use crate::common::http::listen_slowly;
use std::time::Duration;

/// A call to a server that does not answer ends with a network error at the call timeout, instead of
/// hanging. The test sets a short timeout; the production value is the one of the other providers.
#[tokio::test]
async fn call_timeout() {
    let (port, _) = listen_slowly(Duration::from_secs(30), |_| Resp::ok(text_answer("late"))).await;
    let bedrock = client(&format!("http://127.0.0.1:{port}")).with_call_timeout(Duration::from_millis(300));

    let started = std::time::Instant::now();
    let r = tokio::time::timeout(Duration::from_secs(10), bedrock.invoke(&call("q")))
        .await
        .expect("the call must end by itself at the call timeout");

    assert!(matches!(&r, Err(ModelError::Network { .. })), "{r:?}");
    assert!(started.elapsed() < Duration::from_secs(10));
}
