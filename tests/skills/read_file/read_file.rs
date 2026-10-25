use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The model reads a file of the folder of a skill.
#[tokio::test]
async fn read_file() {
    let (p, seen) = program(here(), "read_file", model(read("pdf", "references/forms.md"), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(Forms: fill the fields one by one.\n)");
    let first = body_of(&seen.lock().unwrap()[0]);
    let tool = first["tools"].as_array().unwrap().iter().find(|t| t["function"]["name"] == "read_skill_file").unwrap();
    assert_eq!(tool["function"]["parameters"]["properties"]["skill"]["enum"], json!(["pdf", "notes"]));
    assert_eq!(tool["function"]["parameters"]["required"], json!(["skill", "path"]));
}
