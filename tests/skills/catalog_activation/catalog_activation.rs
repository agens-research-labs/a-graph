use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The model sees the names and descriptions and loads the instructions on request.
#[tokio::test]
async fn catalog_activation() {
    let (p, seen) = program(here(), "catalog_activation", model(activate("pdf"), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, format!("R({PDF})"));

    let first = body_of(&seen.lock().unwrap()[0]);
    // the system text of the node, then how to use the skills and the catalog; no instructions yet
    let system = first["messages"][0]["content"].as_str().unwrap();
    assert!(system.starts_with("Be brief.\n\n"), "{system}");
    assert!(system.contains("call the activate_skill tool"), "{system}");
    assert!(system.contains("<name>pdf</name>") && system.contains("<description>Extracts text from PDF files. Use when the user mentions PDFs.</description>"), "{system}");
    assert!(system.contains("<name>notes</name>") && !system.contains("Step 1"), "{system}");
    assert!(system.contains("read them with the read_skill_file tool"), "{system}");
    // the tool names only the skills of the node
    let tool = first["tools"].as_array().unwrap().iter().find(|t| t["function"]["name"] == "activate_skill").unwrap();
    assert_eq!(tool["function"]["parameters"]["properties"]["name"]["enum"], json!(["pdf", "notes"]));
    assert_eq!(tool["function"]["parameters"]["required"], json!(["name"]));
}
