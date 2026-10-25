use crate::common::logging_delivery::*;

/// Queued events go in batches without losing or reordering any.
#[tokio::test]
async fn batches_in_order() {
    let (url, mut rx) = mock_server(vec![], false).await;
    let d = Delivery { max_batch: 10, ..fast() };
    let log = Logger::to_url_with(LogLevel::Info, Redaction { allow_content: false }, &url, vec![], d).unwrap();
    for n in 0..95 {
        log.log(&Event::new(LogLevel::Info, "e").field("n", n));
    }
    log.close().await;

    let got = drain(&mut rx).await;
    assert!(got.len() < 95, "events were not grouped: {} POSTs", got.len());
    assert!(got.iter().all(|(_, b)| b.lines().count() <= 10));
    let ns: Vec<i64> = lines(&got).iter().map(|l| l["n"].as_i64().unwrap()).collect();
    assert_eq!(ns, (0..95).collect::<Vec<_>>());
}
