use crate::common::run_batch::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A request the provider refuses stops the run and names the node.
#[tokio::test]
async fn request_refused() {
    // the second request of the job is the one of `e`
    let (p, _, _) = program(here(), "request_refused", Some("1")).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(e)".to_string())
    );
    assert!(e.message.contains("this request was refused"), "{}", e.message);
}
