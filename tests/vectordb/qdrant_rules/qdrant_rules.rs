use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Qdrant needs url and collection.
#[test]
fn qdrant_rules() {
    assert!(has(here(), "qdrant_rules", "missing_field", "external.vectordb[0].url"));
    assert!(has(here(), "qdrant_rules", "missing_field", "external.vectordb[0].collection"));
}
