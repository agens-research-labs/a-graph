use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A board that is a symbolic link: the write is refused, and the link and the file it points to stay as
/// they were.
#[cfg(unix)]
#[tokio::test]
async fn link_refused() {
    let (link, target) = (
        "target/test-boards/link_refused.md",
        "target/test-boards/link_refused.target.md",
    );
    fresh_copy(here(), "board.md", target);
    let _ = std::fs::remove_file(link);
    std::os::unix::fs::symlink("link_refused.target.md", link).unwrap();
    let m = manager(here(), "link_refused").unwrap();
    let e = m.write("notes", "line", Place::Bottom).await.unwrap_err();
    assert_eq!(e.rule.code(), "io");
    assert!(e.message.contains("is a symbolic link"), "{}", e.message);
    assert!(std::fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(std::fs::read_to_string(target).unwrap(), HEADER);
}
