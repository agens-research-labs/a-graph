use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What runs inside a group is indented with a bar, once per group it is inside of.
#[tokio::test]
async fn groups_indented() {
    let (mut p, _) = program(here(), "groups_indented", echo).await;
    let screen = show(&mut p, VerboseLevel::Steps, true, false);
    p.run("hi").await.unwrap();
    let lines = screen.lines();
    let at = |needle: &str| {
        lines
            .iter()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no `{needle}` in {lines:?}"))
            .clone()
    };
    assert!(at("▶ s").starts_with("▶"), "the first node is not inside a group");
    assert!(at("▶ d").starts_with("│ ▶"), "{:?}", at("▶ d"));
    assert!(
        at("▶ k").starts_with("│ │ ▶"),
        "a node of the linear group inside the parallel one: {:?}",
        at("▶ k")
    );
    assert!(at("┌─ linear").starts_with("│ ┌─"));
}
