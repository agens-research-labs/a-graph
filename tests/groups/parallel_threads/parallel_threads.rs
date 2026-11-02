use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The branches of a parallel group run at the same time on different threads.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_threads() {
    let (p, calls) = blocking_program(here(), "parallel_threads");
    p.run("hi").await.unwrap();

    // d and e at the same time: each started before the other ended. Not a measure of the whole
    // run, which a slow machine stretches past any bound.
    let calls = calls.lock().unwrap();
    let call = |tag: char| calls.iter().find(|c| c.tag == tag).unwrap();
    let (d, e) = (call('D'), call('E'));
    let first = d.start.min(e.start);
    assert!(
        d.start < e.end && e.start < d.end,
        "d and e did not overlap: d {:?}..{:?}, e {:?}..{:?}",
        d.start - first,
        d.end - first,
        e.start - first,
        e.end - first
    );
    assert_ne!(d.thread, e.thread, "d and e were called on the same thread");
}
