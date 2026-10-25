use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The branches of a parallel group run at the same time on different threads.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_threads() {
    let (p, threads) = blocking_program(here(), "parallel_threads");
    let started = Instant::now();
    p.run("hi").await.unwrap();
    let took = started.elapsed();

    // s, i, f, z one after the other (4 x 200 ms), d and e together (200 ms): about 1 second;
    // one after the other they would take 1.2 seconds
    assert!(took < Duration::from_millis(1150), "took {took:?}");
    let seen = threads.lock().unwrap();
    let thread_of = |tag: char| seen.iter().find(|(t, _)| *t == tag).unwrap().1;
    assert_ne!(thread_of('D'), thread_of('E'), "d and e were called on the same thread");
}
