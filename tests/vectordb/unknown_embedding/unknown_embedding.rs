use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A vectordb that cites an embedding that is not declared is an unknown reference.
#[test]
fn unknown_embedding() {
    assert!(has(
        here(),
        "unknown_embedding",
        "unknown_ref",
        "external.vectordb[0].embedding"
    ));
}
