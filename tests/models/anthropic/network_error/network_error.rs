use super::*;

/// An unreachable server is a network error.
#[tokio::test]
async fn network_error() {
    let r = client("http://127.0.0.1:1").invoke(&call("q")).await;
    assert!(matches!(r, Err(ModelError::Network { .. })), "{r:?}");
}
