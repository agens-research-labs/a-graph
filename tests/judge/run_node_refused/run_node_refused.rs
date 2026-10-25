use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Run node refuses a validation node.
#[tokio::test]
async fn run_node_refused() {
    let (p, _) = program(here(), "run_node_refused", approved()).await;
    let e = p.run_node(p.nodes.entity("judge").unwrap(), NodeInput::default()).await.unwrap_err();
    assert_eq!(e.rule.code(), "not_supported");
}
