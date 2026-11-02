//! The blackboard manager: every test is a folder with its own yml files. The boards the
//! tests change are copies made in `target/test-boards/`, so the file of the test stays as it is.

pub use super::{fresh_copy, yml_in};
pub use a_graph::blackboards::{BlackboardManager, Place};
pub use a_graph::loader::load;
pub use a_graph::schema::key;

pub const HEADER: &str = "---\n<description>\nWhat it is.\n<purpose>\nWhat it is for.\n---\n";

pub fn build_text(src: &str) -> Result<BlackboardManager, Vec<(String, String)>> {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    BlackboardManager::build(doc.take(key::BLACKBOARDS)).map_err(|e| {
        e.0.iter()
            .map(|e| (e.rule.code().to_string(), e.path.to_string()))
            .collect()
    })
}

/// The boards of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
pub fn manager(dir: &std::path::Path, name: &str) -> Result<BlackboardManager, Vec<(String, String)>> {
    build_text(&yml_in(dir, name))
}

/// The manager of a yml whose board `notes` is the file `target/test-boards/<case>.md`, a fresh
/// copy of `board.md`.
pub fn file_board(dir: &std::path::Path, case: &str) -> (BlackboardManager, String) {
    let path = format!("target/test-boards/{case}.md");
    fresh_copy(dir, "board.md", &path);
    (manager(dir, case).unwrap(), path)
}
