use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A validation node has the blocks of a node but only what it reads.
#[test]
fn judge_blocks() {
    let at = "graph.nodes[3].validation-node";
    assert!(has(here(), "with_content", "unexpected_field", &format!("{at}.content")));
    assert!(has(here(), "without_prompt", "missing_field", &format!("{at}.resources")));
    assert!(has(here(), "with_tools", "unexpected_field", &format!("{at}.resources.tools")));
    assert!(has(here(), "blackboard_write", "unexpected_field", &format!("{at}.blackboard.write")));
    assert!(has(here(), "blackboard_read", "unexpected_field", &format!("{at}.blackboard.read")));
    assert!(has(here(), "with_output", "unexpected_field", &format!("{at}.message.output")));
    assert!(has(here(), "prompt_unknown", "unknown_ref", &format!("{at}.resources.prompt")));
}
