use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model error comes back with the node and the model.
#[tokio::test]
async fn model_error_named() {
    let (p, _) = program(here(), "model_error_named", Resp::status(500, "{\"error\":\"the model fell over\"}")).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!((e.rule.code(), e.path.to_string()), ("model_failed", "node(n)".to_string()));
    assert!(e.message.contains("the model fell over") && e.message.contains("`m`"), "{}", e.message);
}
