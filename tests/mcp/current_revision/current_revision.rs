use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server of the current revision gives its tools and answers the calls.
#[tokio::test]
async fn current_revision() {
    let (p, seen) = program(here(), "current_revision", model(ask("say", json!({ "text": "hi" })), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(said: hi)");
    // the model was told the tools of the server, with the schema of their parameters
    let first = body_of(&seen.lock().unwrap()[0]);
    let names: Vec<&str> = first["tools"].as_array().unwrap().iter().map(|t| t["function"]["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["say", "count", "boom", "slow", "die"]);
    assert_eq!(first["tools"][0]["function"]["parameters"]["required"], json!(["text"]));
}
