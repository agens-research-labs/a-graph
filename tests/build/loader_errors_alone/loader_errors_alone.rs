use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The errors of the loader come back alone.
#[test]
fn loader_errors_alone() {
    set_env();
    let e = build(&yml(here(), "loader_errors_alone")).err().unwrap();
    assert_eq!(e.rules(), ["bad_value"]);
}
