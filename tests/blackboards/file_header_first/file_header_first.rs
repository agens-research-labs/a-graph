use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file board keeps the header first and places text below it.
#[test]
fn file_header_first() {
    let (m, path) = file_board(here(), "file_header_first");
    m.write("notes", "one", Place::Bottom).unwrap();
    m.write("notes", "two", Place::Bottom).unwrap();
    m.write("notes", "top", Place::Top).unwrap();
    let want = format!("{HEADER}\ntop\n\none\n\ntwo\n");
    assert_eq!(m.read("notes").unwrap(), want);
    assert_eq!(std::fs::read_to_string(path).unwrap(), want, "the file itself is changed");
}
