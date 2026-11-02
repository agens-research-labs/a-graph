use super::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

// --- the manager, from the yml -------------------------------------------------------------

/// The manager is built from the block and serves models by id.
#[test]
fn manager_serves() {
    use a_graph::loader::load;
    use a_graph::schema::key;

    // SAFETY: tests of this file only; the values are never used to call anything.
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "a");
        std::env::set_var("OPENAI_API_KEY", "o");
    }
    let mut doc = load(&own_yml(here())).unwrap();
    let manager = ModelManager::build(doc.take(key::MODELS)).unwrap();
    assert_eq!(manager.len(), 6);
    for id in ["fast", "strong", "smart", "claude_gcp", "claude_azure", "local"] {
        assert!(manager.get(id).is_some(), "{id}");
        assert!(manager.spec(id).is_some(), "{id}");
    }
    assert!(manager.get("ghost").is_none());
    assert!(manager.spec("fast").unwrap().requires_max_tokens());
    assert!(!manager.spec("smart").unwrap().requires_max_tokens());
}
