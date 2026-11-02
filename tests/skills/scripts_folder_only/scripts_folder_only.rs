use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Only the files of the scripts folder can be run.
#[tokio::test]
async fn scripts_folder_only() {
    for (fixture, path) in [
        ("script_outside_a", "references/forms.md"),
        ("script_outside_b", "scripts/../references/forms.md"),
    ] {
        let (p, _) = program(here(), fixture, model(run("pdf", path, &[]), false)).await;
        let out = p.run("go").await.unwrap();
        let said = &items(&out)[0].1;
        assert!(
            said.starts_with("R(error: ") && said.contains("is not a file inside the `scripts` folder of the skill"),
            "{path}: {said}"
        );
    }
}
