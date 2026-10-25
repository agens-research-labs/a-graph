use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A prompt that uses {{chunks}} on a node that reads no vectordb.
#[test]
fn chunks_unbound() {
    let e = build(&yml(here(), "chunks_unbound")).err().unwrap();
    assert_eq!(e.rules(), ["placeholder_unbound"]);
}
