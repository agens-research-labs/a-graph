use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Temporary files left by a process that died before its rename do not fail the next write. The service
/// is always the same process id in its container, so the leftovers carry the id of this process.
#[tokio::test]
async fn leftover_tmp_ignored() {
    let (m, path) = file_board(here(), "leftover_tmp_ignored");
    let leftovers: Vec<String> = (0..1000)
        .map(|i| format!("{path}.{}.{i}.tmp", std::process::id()))
        .collect();
    for l in &leftovers {
        std::fs::write(l, "half").unwrap();
    }
    let written = m.write("notes", "line", Place::Bottom).await;
    for l in &leftovers {
        let _ = std::fs::remove_file(l);
    }
    written.unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), format!("{HEADER}\nline\n"));
}
