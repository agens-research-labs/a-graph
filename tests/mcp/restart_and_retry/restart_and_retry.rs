use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server that ends in the middle of a call is started again and the call repeated.
#[tokio::test]
async fn restart_and_retry() {
    let marker = "target/test-output/mcp_marker";
    let _ = std::fs::remove_file(marker);
    unsafe { std::env::set_var("A_GRAPH_TEST_MARKER", marker) };
    let (p, _) = program(here(), "restart_and_retry", model(ask("die", json!({})), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(back)");
    assert!(std::path::Path::new(marker).exists(), "the first process ended");
}
