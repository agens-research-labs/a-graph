use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A chromadb database builds.
#[test]
fn chromadb_builds() {
    let e = build(here(), "chromadb_builds").unwrap();
    assert!(matches!(
        e.vectordb.get("kb").unwrap().backend,
        VectorBackend::Chromadb { .. }
    ));
}
