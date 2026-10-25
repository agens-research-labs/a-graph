use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The branches of a linear group go one after the other.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn linear_in_order() {
    let (p, _) = blocking_program(here(), "linear_in_order");
    let started = Instant::now();
    p.run("hi").await.unwrap();
    assert!(started.elapsed() >= Duration::from_millis(1200), "took {:?}", started.elapsed());
}
