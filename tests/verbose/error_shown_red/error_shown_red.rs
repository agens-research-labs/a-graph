use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An error of the run is shown in red, after the nodes that ran.
#[tokio::test]
async fn error_shown_red() {
    let (mut p, _) = program(here(), "error_shown_red", |_| Resp::status(500, "broken")).await;
    let screen = show(&mut p, VerboseLevel::Steps, true, true);
    assert!(p.run("hi").await.is_err());
    let text = screen.text();
    assert!(text.contains("\x1b[31m✗ model_failed"), "{text:?}");
}
