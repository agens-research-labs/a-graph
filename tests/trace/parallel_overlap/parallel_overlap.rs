use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The branches of a parallel group are side by side on the timeline: they overlap.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_overlap() {
    let (_, collector) = serve_at(19814, |_| Resp::ok("{}")).await;
    let (p, _) = program(here(), "parallel_overlap", slow).await;
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    let (i, d) = (named(&spans, "node i"), named(&spans, "node d"));
    assert!(start(d) < end(i) || start(i) < end(d), "the two branches ran together");
    let group = named(&spans, "parallel");
    assert!(end(group) - start(group) >= 300_000_000, "the group lasted at least as long as its slowest branch");
}
