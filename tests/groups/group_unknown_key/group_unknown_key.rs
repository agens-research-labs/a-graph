use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An unknown key in a group or a step is refused.
#[test]
fn group_unknown_key() {
    assert!(!errors(here(), "in_group").is_empty());
    assert!(!errors(here(), "in_step").is_empty());
}
