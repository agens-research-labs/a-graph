use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that reads a vectordb and whose prompt never uses {{chunks}}.
#[test]
fn chunks_unused() {
    let e = build(&yml(here(), "chunks_unused")).err().unwrap();
    assert_eq!(e.rules(), ["placeholder_unused"]);
}
