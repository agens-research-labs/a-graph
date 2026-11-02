use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The prompt of the summary is the one of the yml, with {{synt}} and {{chat}} filled.
#[tokio::test]
async fn summary_own_prompt() {
    let path = "target/test-memory/summary_own_prompt.json";
    clean_output(path);
    let (model_port, seen) = listen(|_| Resp::ok(chat("S", None))).await;
    let p = build(&yml_with(here(), "summary_own_prompt", &[("model", model_port)])).unwrap_or_else(|e| panic!("{e}"));
    p.run_node(entity(&p), user("question")).await.unwrap();
    let seen = seen.lock().unwrap();
    let asked: Value = serde_json::from_str(&seen[1].body).unwrap();
    assert_eq!(asked["options"]["num_predict"], 77, "the limit of the yml");
    let messages = asked["messages"].as_array().unwrap();
    assert_eq!(
        (messages[0]["content"].as_str(), messages[1]["content"].as_str()),
        (Some("Short summary."), Some("OLD= NEW=user: question\nassistant: S"))
    );
}
