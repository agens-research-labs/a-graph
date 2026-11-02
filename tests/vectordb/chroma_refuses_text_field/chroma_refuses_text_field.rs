use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The chromadb database has a `text_field`.
#[test]
fn chroma_refuses_text_field() {
    assert!(has(
        here(),
        "chroma_refuses_text_field",
        "unexpected_field",
        "external.vectordb[0].text_field"
    ));
}
