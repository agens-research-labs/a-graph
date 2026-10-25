use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A script that fails tells the model what it wrote on the error output.
#[tokio::test]
async fn script_fails() {
    let (p, _) = program(here(), "script_fails", model(run("pdf", "scripts/fail.sh", &[]), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: it broke)");
}
