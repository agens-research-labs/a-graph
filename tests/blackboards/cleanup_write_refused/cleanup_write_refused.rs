use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that writes to its board and cleans it in the same task is refused: the write would be lost.
#[test]
fn cleanup_write_refused() {
    std::fs::create_dir_all("target/test-boards").unwrap();
    std::fs::write(
        "target/test-boards/cleanup_write_refused.md",
        "---\n<description>\nWhat it is.\n<purpose>\nWhat it is for.\n---\n",
    )
    .unwrap();
    let e = build(&yml(here(), "cleanup_write_refused")).err().unwrap();
    assert_eq!(e.rules(), ["cleanup_write"]);
}
