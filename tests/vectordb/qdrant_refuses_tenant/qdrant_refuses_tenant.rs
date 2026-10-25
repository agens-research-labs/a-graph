use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `tenant` and `database` belong to chromadb: a qdrant database refuses them.
#[test]
fn qdrant_refuses_tenant() {
    assert!(has(here(), "qdrant_refuses_tenant", "unexpected_field", "external.vectordb[0].tenant"));
    assert!(has(here(), "qdrant_refuses_tenant", "unexpected_field", "external.vectordb[0].database"));
}
