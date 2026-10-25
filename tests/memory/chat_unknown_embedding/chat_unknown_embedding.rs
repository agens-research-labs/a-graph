use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `chat.embedding` cites an embedding that is not declared.
#[test]
fn chat_unknown_embedding() {
    assert!(has(here(), "chat_unknown_embedding", "unknown_ref", "external.memory[0].chat.embedding"));
}
