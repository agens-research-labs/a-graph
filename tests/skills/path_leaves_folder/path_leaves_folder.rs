use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A path that leaves the folder is refused to the model.
#[tokio::test]
async fn path_leaves_folder() {
    for (fixture, path, why) in [("outside_a", "../notes/SKILL.md", "is not a path inside the skill folder"), ("outside_b", "/etc/passwd", "is not a path inside the skill folder"), ("outside_c", "leak.md", "leads outside the skill folder")] {
        let (p, _) = program(here(), fixture, model(read("pdf", path), false)).await;
        let out = p.run("go").await.unwrap();
        let said = &items(&out)[0].1;
        assert!(said.starts_with("R(error: ") && said.contains(why), "{path}: {said}");
    }
}
