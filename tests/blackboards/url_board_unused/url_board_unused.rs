use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A url board is built but cannot be used yet.
#[test]
fn url_board_unused() {
    let m = manager(here(), "url_board_unused").unwrap();
    assert_eq!(m.read("notes").unwrap_err().rule.code(), "not_supported");
}
