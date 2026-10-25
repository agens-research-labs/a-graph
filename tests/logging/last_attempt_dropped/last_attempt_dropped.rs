use crate::common::logging_delivery::*;

/// After the last attempt the batch is dropped and the next one still goes.
#[tokio::test]
async fn last_attempt_dropped() {
    let (url, mut rx) = mock_server(vec![500, 500, 500], false).await;
    let log = Logger::to_url_with(LogLevel::Info, Redaction { allow_content: false }, &url, vec![], fast()).unwrap();
    log.log(&Event::new(LogLevel::Info, "lost"));
    tokio::time::sleep(Duration::from_millis(150)).await; // let the first batch use its 3 attempts
    log.log(&Event::new(LogLevel::Info, "kept"));
    log.close().await;
    let got = drain(&mut rx).await;
    assert_eq!(got.len(), 4);
    assert!(got[3].1.contains("kept"));
}
