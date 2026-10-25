use crate::common::logging_delivery::*;

/// A 4xx is final and is not retried.
#[tokio::test]
async fn client_error_final() {
    let (url, mut rx) = mock_server(vec![401], false).await;
    let log = Logger::to_url_with(LogLevel::Info, Redaction { allow_content: false }, &url, vec![], fast()).unwrap();
    log.log(&Event::new(LogLevel::Info, "x"));
    log.close().await;
    assert_eq!(drain(&mut rx).await.len(), 1);
}
