use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `steps` shows the run, every node and every group, and none of the calls inside.
#[tokio::test]
async fn steps_shows_nodes() {
    let (mut p, _) = program(here(), "steps_shows_nodes", echo).await;
    let screen = show(&mut p, VerboseLevel::Steps, true, false);
    p.run("hi").await.unwrap();
    let text = screen.text();
    for wanted in ["run started", "▶ s · m", "✓ s · ", "┌─ parallel · 2 branches", "└─ parallel done", "┌─ linear · 1 branches", "run finished"] {
        assert!(text.contains(wanted), "`{wanted}` is missing in:\n{text}");
    }
    assert!(!text.contains("model m ·") && !text.contains("prompt:"), "the calls are for `detail`:\n{text}");
}
