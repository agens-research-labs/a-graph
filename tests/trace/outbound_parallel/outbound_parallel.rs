use crate::common::http::listen_slowly;
use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn outbound_parallel() {
    let (first, _) = listen_slowly(Duration::from_millis(25), echo).await;
    let (second, _) = listen_slowly(Duration::from_millis(25), echo).await;
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let source = yml_with(
        here(),
        "outbound_parallel",
        &[("collector", port), ("first", first), ("second", second)],
    );
    let p = Arc::new(build(&source).unwrap());
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    for call in spans.iter().filter(|s| s["kind"] == 3) {
        let node = attribute(call, "a_graph.node").unwrap();
        let port = if node["stringValue"] == "b" { second } else { first };
        assert_eq!(
            attribute(call, "server.port"),
            Some(json!({"intValue": port.to_string()}))
        );
        assert_eq!(
            attribute(call, "http.response.status_code"),
            Some(json!({"intValue": "200"}))
        );
        assert!(attribute(call, "a_graph.http_request_count").is_none());
    }
    assert_eq!(spans.iter().filter(|s| s["kind"] == 3).count(), 6);
}
