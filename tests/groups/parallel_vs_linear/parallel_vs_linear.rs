use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Parallel branches go together and linear ones do not.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_vs_linear() {
    let (p, _) = program(here(), "parallel_slow", slow).await;
    let started = Instant::now();
    p.run("hi").await.unwrap();
    let parallel = started.elapsed();

    let (p, _) = program(here(), "linear_slow", slow).await;
    let started = Instant::now();
    p.run("hi").await.unwrap();
    let linear = started.elapsed();

    assert!(parallel < Duration::from_millis(550), "parallel took {parallel:?}");
    assert!(linear >= Duration::from_millis(600), "linear took {linear:?}");
}
