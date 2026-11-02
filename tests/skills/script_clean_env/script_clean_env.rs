use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A script does not get the environment of the program.
#[tokio::test]
async fn script_clean_env() {
    unsafe { std::env::set_var("ANTHROPIC_API_KEY", "a-secret-key") };
    let (p, _) = program(
        here(),
        "script_clean_env",
        model(run("pdf", "scripts/env.sh", &[]), false),
    )
    .await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(key: none\n)");
}
