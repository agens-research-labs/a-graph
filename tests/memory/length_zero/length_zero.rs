use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `synt.length` and `chat.length` of 0.
#[test]
fn length_zero() {
    assert!(has(
        here(),
        "length_zero",
        "bad_value",
        "external.memory[0].synt.length"
    ));
    assert!(has(
        here(),
        "length_zero",
        "bad_value",
        "external.memory[0].synt.max_tokens"
    ));
    assert!(has(
        here(),
        "length_zero",
        "bad_value",
        "external.memory[0].chat.length"
    ));
}
