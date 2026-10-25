use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An approved message does not stop the run with on error stop.
#[tokio::test]
async fn approved_stop() {
    let (p, seen) = program(here(), "approved_stop", approved()).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(order(&seen), "SJZE");
    assert_eq!(items(&out).len(), 3);
}
