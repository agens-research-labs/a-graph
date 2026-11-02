use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The qdrant database has no `text_field`.
#[test]
fn qdrant_needs_text_field() {
    assert!(has(
        here(),
        "qdrant_needs_text_field",
        "missing_field",
        "external.vectordb[0].text_field"
    ));
}
