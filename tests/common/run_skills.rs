//! The skills of a node. Every test is a folder with its own graph and, under `files/`,
//! its own skills; the model is a mock server like in the tools.

pub use super::http::{Req, Resp, listen};
pub use super::run::{yml_in, yml_with};
pub use super::toolmock::{body_of, items, model};
pub use a_graph::run::{Program, build};
pub use serde_json::{Value, json};
pub use std::sync::{Arc, Mutex};

pub async fn program(
    dir: &std::path::Path,
    name: &str,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    let (port, seen) = listen(handler).await;
    let yml = yml_with(dir, name, &[("model", port)]);
    (Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

pub fn activate(name: &str) -> Vec<(&'static str, Value)> {
    vec![("activate_skill", json!({ "name": name }))]
}

pub const PDF: &str = "<skill_content name=\"pdf\">\n# PDF\n\nStep 1: read the file.\n\nThe paths below are relative to the folder of the skill; read a file with the read_skill_file tool.\n<skill_resources>\n  <file>assets/blob.bin</file>\n  <file>references/forms.md</file>\n  <file>scripts/args.sh</file>\n  <file>scripts/env.sh</file>\n  <file>scripts/extract.sh</file>\n  <file>scripts/fail.sh</file>\n  <file>scripts/noexec.sh</file>\n  <file>scripts/slow.sh</file>\n</skill_resources>\n</skill_content>";

pub fn read(skill: &str, path: &str) -> Vec<(&'static str, Value)> {
    vec![("read_skill_file", json!({ "skill": skill, "path": path }))]
}

/// A skill folder made under target/: a big file and more files than are listed.
pub fn big_skill(dir: &str) {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(format!("{dir}/many")).unwrap();
    std::fs::write(
        format!("{dir}/SKILL.md"),
        "---\nname: pdf\ndescription: big\n---\nbody\n",
    )
    .unwrap();
    std::fs::write(format!("{dir}/big.txt"), vec![b'a'; 100_001]).unwrap();
    for i in 0..205 {
        std::fs::write(format!("{dir}/many/f{i:03}.txt"), "x").unwrap();
    }
}

pub fn run(skill: &str, path: &str, args: &[&str]) -> Vec<(&'static str, Value)> {
    vec![(
        "run_skill_script",
        json!({ "skill": skill, "path": path, "args": args }),
    )]
}
