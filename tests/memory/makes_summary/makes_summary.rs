use crate::common::run_node::*;
use crate::common::fresh_copy;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// At the second exchange the summary is made with the old summary and the exchanges, and `chat` is emptied.
#[tokio::test]
async fn makes_summary() {
    let path = "target/test-memory/makes_summary.json";
    fresh_copy(here(), "files/session.json", path);
    let (_, seen) = serve_at(19606, |r| {
        let body: Value = serde_json::from_str(&r.body).unwrap();
        let text = body["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap().to_string();
        Resp::ok(chat(if text.starts_with("Previous summary") { "THE SUMMARY" } else { "a2" }, None))
    })
    .await;
    let p = build(&yml_in(here(), "makes_summary")).unwrap_or_else(|e| panic!("{e}"));
    p.run_node(entity(&p), user("q2")).await.unwrap();

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2, "the node, then the summary");
    let asked: Value = serde_json::from_str(&seen[1].body).unwrap();
    let user_text = asked["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap();
    assert_eq!(user_text, "Previous summary:\nEARLIER\n\nNew exchanges:\nuser: q1\nassistant: a1\nuser: q2\nassistant: a2");
    assert_eq!(asked["options"]["num_predict"], 4000, "the default limit of a summary");
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!((records[0]["synt"].as_str(), records[0]["chat"].as_array().unwrap().len()), (Some("THE SUMMARY"), 0));
}
