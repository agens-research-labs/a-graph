use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With colors the nodes are bold cyan and the groups bold orange; without, the same text has no escape codes.
#[tokio::test]
async fn colors_on_terminal() {
    let (mut p, _) = program(here(), "colors_on_terminal", echo).await;
    let screen = show(&mut p, VerboseLevel::Steps, true, true);
    p.run("hi").await.unwrap();
    let text = screen.text();
    assert!(text.contains("\x1b[1;36m▶ s\x1b[0m"), "{text:?}");
    assert!(text.contains("\x1b[1;38;5;208m┌─ parallel"), "{text:?}");

    let (mut q, _) = program(here(), "colors_on_terminal_plain", echo).await;
    let plain = show(&mut q, VerboseLevel::Steps, true, false);
    q.run("hi").await.unwrap();
    assert!(!plain.text().contains('\x1b'));
}
