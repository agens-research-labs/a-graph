use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file of a skill the node does not have is told to the model.
#[tokio::test]
async fn read_unknown_skill() {
    let (p, _) = program(here(), "read_unknown_skill", model(read("nope", "x"), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: this node has no skill called `nope`)");
}
