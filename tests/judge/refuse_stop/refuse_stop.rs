use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A refused message with on error stop ends the run without reaching end.
#[tokio::test]
async fn refuse_stop() {
    let (p, seen) = program(here(), "refuse_stop", refused()).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(order(&seen), "SJ");
    // the output says why the run ended
    assert_eq!(items(&out), [pair("s", "S(hi)"), pair("judge", "reasons: too long\nlog: checked the length")]);
}
