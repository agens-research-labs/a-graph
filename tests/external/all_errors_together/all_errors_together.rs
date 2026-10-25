use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Errors of all managers are reported together.
#[test]
fn all_errors_together() {
    assert!(build(here(), "all_errors_together").unwrap_err().len() >= 2);
}
