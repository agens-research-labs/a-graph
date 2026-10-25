use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Every error is reported not only the first.
#[test]
fn all_errors() {
    assert!(errors(here(), "all_errors").len() >= 2);
}
