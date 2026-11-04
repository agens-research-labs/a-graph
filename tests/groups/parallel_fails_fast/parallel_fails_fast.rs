use crate::common::http::listen_held;
use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The first branch that fails ends the run, even when a branch before it in the fan is still going:
/// the answer of `d` is held for ever, `e` fails, and the run must end with the error of `e`.
#[tokio::test]
async fn parallel_fails_fast() {
    let (port, seen, _release) = listen_held(
        |req| user_of(req).starts_with('D'),
        |req| {
            if user_of(req).starts_with('E') {
                Resp::status(500, "{\"error\":\"fell over\"}")
            } else {
                echo(req)
            }
        },
    )
    .await;
    let yml = yml_with(here(), "parallel_fails_fast", &[("model", port)]);
    let p = Shared::new(build(&yml).unwrap_or_else(|e| panic!("{e}")));

    // `d` never answers: only a run that does not wait for it ends within the bound
    let ended = tokio::time::timeout(Duration::from_secs(10), p.run("hi")).await;
    let asked = order(&seen);
    assert!(asked.contains('D') && asked.contains('E'), "the mock was asked {asked}");
    let e = ended.expect("the run waited for `d` after `e` had failed").unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(e)".to_string())
    );
}
