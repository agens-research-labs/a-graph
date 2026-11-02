use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program that takes too long has failed.
#[tokio::test]
async fn stdio_timeout() {
    let (p, _) = program(here(), "stdio_timeout", model(city(), false)).await;
    let started = std::time::Instant::now();
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: the tool did not answer within 1 s)");
    // the program was given up after the timeout, not waited for (it sleeps 5 seconds)
    assert!(
        started.elapsed() < std::time::Duration::from_secs(4),
        "{:?}",
        started.elapsed()
    );
}
