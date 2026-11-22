use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `detail` shows every checked answer: its outcome and which attempt it was.
#[tokio::test]
async fn schema_attempt_shown() {
    let (mut p, _) = program(here(), "schema_attempt_shown", |_| Resp::ok(chat("{\"a\": 1}"))).await;
    let screen = show(&mut p, VerboseLevel::Detail, true, false);
    p.run("hi").await.unwrap();
    let lines = screen.lines();
    assert!(
        lines.iter().any(|l| l.contains("structured valid · api · attempt 1/1")),
        "{lines:?}"
    );
}
