use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The prompt of a summary from a url is downloaded when the run starts and used for the summary; a text without
/// {{chat}} stops the run before the first node, as the check of the file does for a prompt written in the yml.
#[tokio::test]
async fn summary_url_prompt() {
    let path = "target/test-memory/summary_url_prompt.json";
    clean_output(path);
    let (files, _) =
        listen(|_| Resp::ok("content:\n  system: \"Short summary.\"\n  user: \"OLD={{synt}} NEW={{chat}}\"\n")).await;
    let (p, seen) = program_with(
        here(),
        "summary_url_prompt",
        &[("files", files)],
        Resp::ok(chat("S", None)),
    )
    .await;
    Arc::new(p).run("question").await.unwrap();
    let asked: Value = serde_json::from_str(&seen.lock().unwrap()[1].body).unwrap();
    let messages = asked["messages"].as_array().unwrap();
    assert_eq!(
        (messages[0]["content"].as_str(), messages[1]["content"].as_str()),
        (Some("Short summary."), Some("OLD= NEW=user: question\nassistant: S"))
    );

    clean_output(path);
    let (files, _) = listen(|_| Resp::ok("content:\n  system: \"Short summary.\"\n  user: \"OLD={{synt}}\"\n")).await;
    let (p, seen) = program_with(
        here(),
        "summary_url_prompt",
        &[("files", files)],
        Resp::ok(chat("S", None)),
    )
    .await;
    let e = Arc::new(p).run("question").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string().as_str()),
        ("placeholder_unused", "external.memory[0].synt.prompt"),
        "{e}"
    );
    assert!(seen.lock().unwrap().is_empty(), "the model is never called");
}
