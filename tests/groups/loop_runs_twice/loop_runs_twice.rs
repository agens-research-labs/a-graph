use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that is both `start` and `end` is a loop: it runs, then it runs again.
#[tokio::test]
async fn loop_runs_twice() {
    let (p, seen) = program(here(), "loop_runs_twice", echo).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out).len(), 2);
    assert_eq!(seen.lock().unwrap().len(), 2, "two calls to the model");
}
