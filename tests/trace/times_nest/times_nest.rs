use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A span begins and ends inside the one it hangs from, and no span ends before it begins.
#[tokio::test]
async fn times_nest() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(here(), "times_nest", &[("collector", collector_port)], echo).await;
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    for s in &spans {
        assert!(end(s) >= start(s), "{}", s["name"]);
        if let Some(parent) = s["parentSpanId"].as_str() {
            let p = spans.iter().find(|x| x["spanId"] == parent).unwrap();
            assert!(start(s) >= start(p) && end(s) <= end(p), "`{}` is not inside `{}`", s["name"], p["name"]);
        }
    }
}
