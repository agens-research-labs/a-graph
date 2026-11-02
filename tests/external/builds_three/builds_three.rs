use crate::common::external::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Valid file builds the three managers.
#[test]
fn builds_three() {
    let e = build_text(&own_yml(here())).unwrap_or_else(|e| panic!("{e:?}"));
    let kb = e.vectordb.get("kb").unwrap();
    assert!(matches!(&kb.backend, VectorBackend::Qdrant { collection, .. } if collection == "docs"));
    assert_eq!(
        kb.api_key.as_deref(),
        Some("QDRANT_API_KEY"),
        "the name of the variable, not a value"
    );
    assert_eq!(kb.top_k, Some(5));
    assert!(
        matches!(&e.memory.get("session").unwrap().backend, MemoryBackend::JsonFile { path } if path == "./s.json")
    );
    assert!(matches!(&e.memory.get("vmem").unwrap().backend, MemoryBackend::Vectordb { id } if id == "kb"));
}
