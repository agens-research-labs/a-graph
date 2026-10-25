use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A script of a skill the node does not have is told to the model.
#[tokio::test]
async fn script_unknown_skill() {
    let (p, _) = program(here(), "script_unknown_skill", model(run("nope", "scripts/x.sh", &[]), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: this node has no skill called `nope`)");
}
