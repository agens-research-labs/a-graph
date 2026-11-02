use crate::common::logging_delivery::*;

/// Url output delivers events in order with headers level and redaction.
#[tokio::test]
async fn url_delivery() {
    let (url, mut rx) = mock_server(vec![], false).await;
    let log = Logger::to_url(
        LogLevel::Info,
        Redaction { allow_content: false },
        &url,
        vec![("Authorization".into(), "Bearer t".into())],
    )
    .unwrap();
    log.log(&Event::new(LogLevel::Info, "first").field("n", 1));
    log.log(&Event::new(LogLevel::Debug, "dropped"));
    log.log(&Event::new(LogLevel::Error, "second").content("response", "secret"));
    log.close().await;

    let got = drain(&mut rx).await;
    assert!(got[0].0.contains("authorization: bearer t") && got[0].0.contains("application/x-ndjson"));
    let l = lines(&got);
    assert_eq!(l.len(), 2);
    assert_eq!((l[0]["code"].as_str(), l[0]["n"].as_i64()), (Some("first"), Some(1)));
    assert_eq!(l[1]["code"], "second");
    assert_eq!(l[1]["content_redacted"], true);
}
