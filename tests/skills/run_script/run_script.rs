use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The model runs a script of a skill and gets what it wrote.
#[tokio::test]
async fn run_script() {
    let (p, seen) = program(here(), "run_script", model(run("pdf", "scripts/extract.sh", &[]), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(extracting\n)");
    // the model is told it can, and which skills
    let first = body_of(&seen.lock().unwrap()[0]);
    assert!(first["messages"][0]["content"].as_str().unwrap().contains("run it with the run_skill_script tool"));
    let tool = first["tools"].as_array().unwrap().iter().find(|t| t["function"]["name"] == "run_skill_script").unwrap();
    assert_eq!(tool["function"]["parameters"]["properties"]["skill"]["enum"], json!(["pdf", "notes"]));
    assert_eq!(tool["function"]["parameters"]["required"], json!(["skill", "path"]));
}
