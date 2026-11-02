use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An error of a node stops the run and names the node.
#[tokio::test]
async fn node_error_named() {
    let count = Arc::new(AtomicUsize::new(0));
    let (p, seen) = program(here(), "node_error_named", move |req| {
        if count.fetch_add(1, Ordering::SeqCst) == 1 {
            Resp::status(500, "{\"error\":\"fell over\"}")
        } else {
            echo(req)
        }
    })
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(b)".to_string())
    );
    assert_eq!(seen.lock().unwrap().len(), 2, "c never ran");
}
