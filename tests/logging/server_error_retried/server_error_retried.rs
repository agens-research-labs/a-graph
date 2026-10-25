use crate::common::logging_delivery::*;

/// A 5xx is retried and the batch arrives once it succeeds.
#[tokio::test]
async fn server_error_retried() {
    let (url, mut rx) = mock_server(vec![500, 503], false).await;
    let log = Logger::to_url_with(LogLevel::Info, Redaction { allow_content: false }, &url, vec![], fast()).unwrap();
    log.log(&Event::new(LogLevel::Info, "x"));
    log.close().await;
    let got = drain(&mut rx).await;
    assert_eq!(got.len(), 3, "two failures and the successful attempt");
    assert!(got.iter().all(|(_, b)| b == &got[0].1));
}
