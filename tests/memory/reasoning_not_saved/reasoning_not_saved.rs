use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Only the answer is kept: the reasoning of the model is neither saved in the file nor given to the summary.
#[tokio::test]
async fn reasoning_not_saved() {
    let path = "target/test-memory/reasoning_not_saved.json";
    clean_output(path);
    let (model_port, seen) = listen(|r| {
        let body: Value = serde_json::from_str(&r.body).unwrap();
        let text = body["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap().to_string();
        Resp::ok(chat(if text.starts_with("Previous summary") { "THE SUMMARY" } else { "the answer" }, Some("SECRET THOUGHTS")))
    })
    .await;
    let p = build(&yml_with(here(), "reasoning_not_saved", &[("model", model_port)])).unwrap_or_else(|e| panic!("{e}"));
    p.run_node(entity(&p), user("the question")).await.unwrap();

    // the summary call got the exchange without the thoughts
    let asked = seen.lock().unwrap()[1].body.clone();
    assert!(asked.contains("assistant: the answer") && !asked.contains("SECRET THOUGHTS"), "{asked}");
    // the file holds the summary and nothing of the thoughts
    let file = std::fs::read_to_string(path).unwrap();
    assert!(file.contains("THE SUMMARY") && !file.contains("SECRET THOUGHTS"), "{file}");
}
