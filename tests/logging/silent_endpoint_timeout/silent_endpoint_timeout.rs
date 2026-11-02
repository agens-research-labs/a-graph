use crate::common::logging_delivery::*;

/// A silent endpoint hits the timeout and close still returns.
#[tokio::test]
async fn silent_endpoint_timeout() {
    let (url, mut rx) = mock_server(vec![], true).await;
    let log = Logger::to_url_with(LogLevel::Info, Redaction { allow_content: false }, &url, vec![], fast()).unwrap();
    log.log(&Event::new(LogLevel::Info, "x"));
    let started = std::time::Instant::now();
    log.close().await;
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "close took {:?}",
        started.elapsed()
    );
    assert_eq!(drain(&mut rx).await.len(), 3, "1 attempt + 2 retries");
}
