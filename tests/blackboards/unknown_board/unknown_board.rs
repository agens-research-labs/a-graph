use crate::common::blackboards::*;
use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An unknown board is an execution error.
#[test]
fn unknown_board() {
    let m = build_text(&own_yml(here())).unwrap();
    assert_eq!(m.read("other").unwrap_err().rule.code(), "unknown_ref");
}
