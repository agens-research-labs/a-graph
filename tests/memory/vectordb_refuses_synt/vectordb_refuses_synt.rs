use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory of provider vectordb with `synt`.
#[test]
fn vectordb_refuses_synt() {
    assert!(has(here(), "vectordb_refuses_synt", "unexpected_field", "external.memory[1].synt"));
}
