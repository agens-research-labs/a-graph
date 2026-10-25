use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Media output is text only for bedrock.
#[test]
fn bedrock_text_only() {
    assert!(has(here(), "bedrock_text_only", "output_not_text", "models[1].output"));
}
