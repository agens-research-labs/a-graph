use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A vectordb must say which embedding it uses.
#[test]
fn missing_embedding() {
    assert!(has(
        here(),
        "missing_embedding",
        "missing_field",
        "external.vectordb[0].embedding"
    ));
}
