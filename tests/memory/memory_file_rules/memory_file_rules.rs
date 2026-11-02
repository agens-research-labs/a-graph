use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Json file memory needs a path and refuses a ref.
#[test]
fn memory_file_rules() {
    assert!(has(
        here(),
        "memory_file_rules",
        "missing_field",
        "external.memory[0].path"
    ));
    assert!(has(
        here(),
        "memory_file_rules",
        "unexpected_field",
        "external.memory[0].ref"
    ));
}
