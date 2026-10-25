use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Hosts are compared in lowercase.
#[test]
fn hosts_lowercase() {
    assert_eq!(errors(here(), "hosts_lowercase"), vec![]);
}
