use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A call that takes longer than the timeout has failed.
#[tokio::test]
async fn call_timeout() {
    let (p, _) = program(here(), "call_timeout", model(ask("slow", json!({})), false)).await;
    let started = std::time::Instant::now();
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: the tool did not answer within 1 s)");
    // the tool takes ten seconds: the run does not wait for it
    assert!(
        started.elapsed() < std::time::Duration::from_secs(6),
        "{:?}",
        started.elapsed()
    );
}
