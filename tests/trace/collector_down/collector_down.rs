use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A collector that does not take the trace does not stop the run: the send is retried and then given up.
#[tokio::test]
async fn collector_down() {
    let (_, collector) = serve_at(19817, |_| Resp::status(500, "down")).await;
    let (p, _) = program(here(), "collector_down", echo).await;
    assert!(p.run("hi").await.is_ok());
    assert_eq!(collector.lock().unwrap().len(), 4, "the first try and three more");
}
