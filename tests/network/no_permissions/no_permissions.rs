use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Absent permissions allow nothing.
#[test]
fn no_permissions() {
    let e = errors(here(), "no_permissions");
    assert!(!e.is_empty() && e.iter().all(|(r, _)| r == "network_not_allowed"), "{e:?}");
}
