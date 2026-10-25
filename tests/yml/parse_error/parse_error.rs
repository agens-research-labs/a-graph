use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Yml that is not well formed is a parse error.
#[test]
fn parse_error() {
    assert_eq!(errors(here(), "parse_error")[0].0, "parse");
}
