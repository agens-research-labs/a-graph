use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A concurrent group lets in only limit branches at a time.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_limit() {
    // limit 1: d and e one after the other, like a linear group
    let (p, _) = blocking_program(here(), "concurrent_limit");
    let started = Instant::now();
    p.run("hi").await.unwrap();
    assert!(started.elapsed() >= Duration::from_millis(1200), "took {:?}", started.elapsed());
}
