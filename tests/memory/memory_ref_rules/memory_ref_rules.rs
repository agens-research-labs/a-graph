use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Vectordb memory needs a ref and refuses a path.
#[test]
fn memory_ref_rules() {
    assert!(has(
        here(),
        "memory_ref_rules",
        "missing_field",
        "external.memory[1].ref"
    ));
    assert!(has(
        here(),
        "memory_ref_rules",
        "unexpected_field",
        "external.memory[1].path"
    ));
}
