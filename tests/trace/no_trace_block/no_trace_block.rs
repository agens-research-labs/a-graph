use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Without a `trace` block there is no tracer and nothing is sent.
#[tokio::test]
async fn no_trace_block() {
    let (_, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program(here(), "no_trace_block", echo).await;
    assert!(p.tracer.is_none());
    p.run("hi").await.unwrap();
    assert!(collector.lock().unwrap().is_empty());
}
