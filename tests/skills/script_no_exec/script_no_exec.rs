use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A script without permission to execute says so.
#[tokio::test]
async fn script_no_exec() {
    let (p, _) = program(
        here(),
        "script_no_exec",
        model(run("pdf", "scripts/noexec.sh", &[]), false),
    )
    .await;
    let out = p.run("go").await.unwrap();
    assert_eq!(
        items(&out)[0].1,
        "R(error: `noexec.sh` cannot be run: it has no permission to execute)"
    );
}
