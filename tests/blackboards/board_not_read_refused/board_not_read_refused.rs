use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that does not read its board has no placeholder for it: a prompt that uses it is refused.
#[test]
fn board_not_read_refused() {
    std::fs::create_dir_all("target/test-boards").unwrap();
    std::fs::write(
        "target/test-boards/board_not_read_refused.md",
        "---\n<description>\nWhat it is.\n<purpose>\nWhat it is for.\n---\n",
    )
    .unwrap();
    let e = build(&yml(here(), "board_not_read_refused")).err().unwrap();
    assert_eq!(e.rules(), ["placeholder_unbound"]);
}
