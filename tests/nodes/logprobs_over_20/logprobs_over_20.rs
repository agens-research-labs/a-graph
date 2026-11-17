use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The alternatives of a token are from 0 to 20: more, or a negative number, is refused.
#[test]
fn logprobs_over_20() {
    let at = "graph.nodes[0].message.output.logprobs";
    assert!(
        has(here(), "over_20", "bad_value", at),
        "{:?}",
        errors(here(), "over_20")
    );
    assert!(
        has(here(), "negative", "bad_value", at),
        "{:?}",
        errors(here(), "negative")
    );
}
