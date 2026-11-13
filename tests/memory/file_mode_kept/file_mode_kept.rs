use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory file the operator set to 0600 is still 0600 after a write, not the mode of a new file.
#[cfg(unix)]
#[tokio::test]
async fn file_mode_kept() {
    use std::os::unix::fs::PermissionsExt;
    let path = "target/test-memory/file_mode_kept.json";
    clean_output(path);
    std::fs::write(path, "[]").unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let (p, _) = program(here(), "file_mode_kept", Resp::ok(chat("the answer", None))).await;
    p.run_node(entity(&p), user("the question")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(records.as_array().unwrap().len(), 1);
    let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "mode {mode:o}");
}
