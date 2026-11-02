use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Memory ref to a vectordb is linked.
#[test]
fn memory_ref_linked() {
    assert!(has(
        here(),
        "memory_ref_linked",
        "unknown_ref",
        "external.memory[1].ref"
    ));
}
