use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The prompt of the summary lacks `{{chat}}`.
#[test]
fn synt_prompt_needs_both() {
    set_env();
    assert!(has(here(), "synt_prompt_needs_both", "placeholder_unused", "external.memory[0].synt.prompt"));
}
