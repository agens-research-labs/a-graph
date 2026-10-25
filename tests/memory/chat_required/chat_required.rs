use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A json_file memory without `chat`.
#[test]
fn chat_required() {
    assert!(has(here(), "chat_required", "missing_field", "external.memory[0].chat"));
}
