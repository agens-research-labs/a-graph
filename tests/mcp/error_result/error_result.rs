use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A result marked as an error tells the model.
#[tokio::test]
async fn error_result() {
    let (p, _) = program(here(), "error_result", model(ask("boom", json!({})), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: it failed)");
}
