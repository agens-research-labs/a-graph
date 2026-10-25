use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Run scripts false takes the tool away for that skill.
#[tokio::test]
async fn run_scripts_false() {
    let (p, seen) = program(here(), "run_scripts_false", model(run("pdf", "scripts/extract.sh", &[]), false)).await;
    let out = p.run("go").await.unwrap();
    // `notes` still runs scripts, so the tool exists, but it names only that skill
    let first = body_of(&seen.lock().unwrap()[0]);
    let tool = first["tools"].as_array().unwrap().iter().find(|t| t["function"]["name"] == "run_skill_script").unwrap();
    assert_eq!(tool["function"]["parameters"]["properties"]["skill"]["enum"], json!(["notes"]));
    assert_eq!(items(&out)[0].1, "R(error: the scripts of the skill `pdf` cannot be run)");
}
