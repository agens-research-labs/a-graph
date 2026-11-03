use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file that lost its header after the check is not written nor cleaned: the change is refused
/// and the file stays as it is.
#[tokio::test]
async fn write_no_header() {
    let (m, path) = file_board(here(), "write_no_header");
    std::fs::write(&path, "no header\n").unwrap();
    let write = m.write("notes", "text", Place::Bottom).await.unwrap_err();
    assert_eq!(write.rule.code(), "blackboard_header");
    let cleanup = m.cleanup("notes").await.unwrap_err();
    assert_eq!(cleanup.rule.code(), "blackboard_header");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "no header\n");
}
