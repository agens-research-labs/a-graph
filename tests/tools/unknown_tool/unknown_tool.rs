use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A tool the node does not have is told to the model.
#[tokio::test]
async fn unknown_tool() {
    let (p, _) = program(here(), "unknown_tool", model(vec![("nope", json!({}))], false)).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: this node has no tool called `nope`)");
}
