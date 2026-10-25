use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Batch is not allowed with publisher anthropic.
#[test]
fn no_batch_anthropic() {
    assert!(has(here(), "vertex", "batch_not_allowed", "models[3].batch"));
    assert!(has(here(), "azure", "batch_not_allowed", "models[4].batch"));
}
