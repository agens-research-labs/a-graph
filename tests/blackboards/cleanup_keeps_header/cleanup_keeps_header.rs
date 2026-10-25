use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Cleanup removes the body and keeps the header.
#[test]
fn cleanup_keeps_header() {
    let (m, _) = file_board(here(), "cleanup_keeps_header");
    m.write("notes", "text", Place::Bottom).unwrap();
    m.cleanup("notes").unwrap();
    assert_eq!(m.read("notes").unwrap(), HEADER);
}
