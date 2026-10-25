use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file without the header is refused.
#[test]
fn file_no_header() {
    assert_eq!(manager(here(), "file_no_header").unwrap_err()[0].0, "blackboard_header");
}
