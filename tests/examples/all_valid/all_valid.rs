/// Every graph in `examples/` is valid when checked from its own folder, where it is meant to be run.
#[test]
fn all_valid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut checked = 0;
    for entry in std::fs::read_dir(&root).unwrap().flatten() {
        let folder = entry.path();
        let graph = folder.join("graph.yml");
        if !graph.exists() {
            continue;
        }
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_a-graph")).arg("--check").arg("graph.yml").current_dir(&folder).output().unwrap();
        assert!(out.status.success(), "{} is not valid:\n{}", graph.display(), String::from_utf8_lossy(&out.stderr));
        checked += 1;
    }
    assert!(checked >= 2, "only {checked} examples found");
}
