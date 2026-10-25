use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Errors of all managers are reported together.
#[test]
fn all_errors_together() {
    assert!(errors(here(), "all_errors_together").len() >= 2);
}
