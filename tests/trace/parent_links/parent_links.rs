use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Every span hangs from the one that contains it: nodes from their group, calls from their node.
#[tokio::test]
async fn parent_links() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(here(), "parent_links", &[("collector", collector_port)], echo).await;
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    let id = |name: &str| named(&spans, name)["spanId"].as_str().unwrap().to_string();
    let parent = |name: &str| named(&spans, name)["parentSpanId"].as_str().map(str::to_string);

    assert_eq!(parent("run"), None, "the run is the root");
    assert_eq!(parent("node s"), Some(id("run")));
    assert_eq!(parent("parallel"), Some(id("run")));
    assert_eq!(parent("node i"), Some(id("parallel")));
    assert_eq!(parent("linear"), Some(id("parallel")), "a group inside a group");
    assert_eq!(parent("node k"), Some(id("linear")));
    let call = spans.iter().find(|s| s["name"] == "model m" && s["parentSpanId"] == json!(id("node k"))).expect("a call inside node k");
    assert!(call["parentSpanId"].is_string());
}
