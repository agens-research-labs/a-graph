use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that reads a vectordb and has no prompt.
#[test]
fn chunks_no_prompt() {
    let e = build(&yml(here(), "chunks_no_prompt")).err().unwrap();
    assert_eq!(e.rules(), ["placeholder_unused"]);
}
