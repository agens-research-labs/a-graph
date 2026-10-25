use crate::common::loader::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Valid file loads and hands blocks over by move.
#[test]
fn valid_loads() {
    let mut doc = load(&own_yml(here())).unwrap_or_else(|e| panic!("{e}"));
    let models = doc.take(key::MODELS).expect("models block");
    assert_eq!(models.as_sequence().map(|s| s.len()), Some(6));
    assert!(doc.take(key::MODELS).is_none(), "a block is handed over once");
}
