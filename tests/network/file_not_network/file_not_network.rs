use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file or stdout is not network.
#[test]
fn file_not_network() {
    assert_eq!(errors(here(), "file_not_network"), vec![]);
}
