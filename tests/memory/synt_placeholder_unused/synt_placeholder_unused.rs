use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory with a summary must be used in the prompt.
#[test]
fn synt_placeholder_unused() {
    let e = build(&yml(here(), "synt_placeholder_unused")).err().unwrap();
    assert_eq!(e.rules(), ["placeholder_unused"]);
}
