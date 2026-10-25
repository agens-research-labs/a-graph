use crate::common::blackboards::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The fixture boards are valid.
#[test]
fn boards_valid() {
    let m = build_text(&own_yml(here())).unwrap();
    assert!(m.ids().any(|i| i == "findings") && m.ids().any(|i| i == "notes"));
}
