use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The chromadb database has neither `url` nor `collection`.
#[test]
fn chroma_needs_url_collection() {
    assert!(has(
        here(),
        "chroma_needs_url_collection",
        "missing_field",
        "external.vectordb[0].url"
    ));
    assert!(has(
        here(),
        "chroma_needs_url_collection",
        "missing_field",
        "external.vectordb[0].collection"
    ));
}
