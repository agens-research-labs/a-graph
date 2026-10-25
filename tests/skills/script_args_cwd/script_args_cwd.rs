use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The arguments are a list and the script runs in the folder of the skill.
#[tokio::test]
async fn script_args_cwd() {
    // an argument with a space, a quote and a `$` reaches the script as it is: there is no shell
    let (p, _) = program(here(), "script_args_cwd", model(run("pdf", "scripts/args.sh", &["a b", "it's", "$HOME"]), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(args: [a b] [it's] [$HOME]\ncwd: pdf\n)");
}
