use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A missing file is an io error.
#[test]
fn file_missing() {
    let e = manager(here(), "file_missing").unwrap_err();
    assert_eq!(e[0], ("io".to_string(), "blackboards[1].source.file".to_string()));
}
