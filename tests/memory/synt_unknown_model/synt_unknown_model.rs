use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `synt.model` cites a model that is not declared.
#[test]
fn synt_unknown_model() {
    assert!(has(here(), "synt_unknown_model", "unknown_ref", "external.memory[0].synt.model"));
}
