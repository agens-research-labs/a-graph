use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A script that takes longer than the timeout of the skill is stopped.
#[tokio::test]
async fn script_timeout() {
    let (p, _) = program(
        here(),
        "script_timeout",
        model(run("pdf", "scripts/slow.sh", &[]), false),
    )
    .await;
    let started = std::time::Instant::now();
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: `slow.sh` did not finish within 1 s)");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(4),
        "{:?}",
        started.elapsed()
    );
}
