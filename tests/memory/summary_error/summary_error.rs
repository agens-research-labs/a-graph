use crate::common::fresh_copy;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the provider wrote when the summary of a memory fails stays out of the message: it is the
/// detail.
#[tokio::test]
async fn summary_error() {
    let path = "target/test-memory/summary_error.json";
    fresh_copy(here(), "files/session.json", path);
    let (model_port, _) = listen(|r| {
        let body: Value = serde_json::from_str(&r.body).unwrap();
        let text = body["messages"].as_array().unwrap().last().unwrap()["content"]
            .as_str()
            .unwrap()
            .to_string();
        if text.starts_with("Previous summary") {
            Resp::status(500, r#"{"error":"the patient has diabetes"}"#)
        } else {
            Resp::ok(chat("a2", None))
        }
    })
    .await;
    let p = build(&yml_with(here(), "summary_error", &[("model", model_port)])).unwrap_or_else(|e| panic!("{e}"));
    let e = p.run_node(entity(&p), user("q2")).await.unwrap_err();
    assert_eq!(e.rule.code(), "model_failed");
    assert!(
        e.message.contains("summary of memory") && !e.message.contains("diabetes"),
        "{}",
        e.message
    );
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));
}
