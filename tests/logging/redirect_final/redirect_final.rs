use crate::common::logging_delivery::*;

/// A 3xx is final and is not retried: redirects are not followed, so it is not an error that may pass.
#[tokio::test]
async fn redirect_final() {
    let (url, mut rx) = mock_server(vec![302], false).await;
    let log = Logger::to_url_with(LogLevel::Info, Redaction { allow_content: false }, &url, vec![], fast()).unwrap();
    log.log(&Event::new(LogLevel::Info, "x"));
    log.close().await;
    assert_eq!(drain(&mut rx).await.len(), 1);
}
