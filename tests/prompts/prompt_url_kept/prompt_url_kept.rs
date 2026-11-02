use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Prompt with a url is kept as a source not read yet.
#[test]
fn prompt_url_kept() {
    let r = build(here(), "prompt_url_kept").unwrap();
    assert!(matches!(
        r.prompts.get("chat").unwrap().body,
        PromptBody::Source(Source::Url { .. })
    ));
}
