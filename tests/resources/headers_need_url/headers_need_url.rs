use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `headers` goes with `url` only.
#[test]
fn headers_need_url() {
    assert!(has(
        here(),
        "headers_need_url",
        "unexpected_field",
        "resources.prompts[1].source.headers"
    ));
}
