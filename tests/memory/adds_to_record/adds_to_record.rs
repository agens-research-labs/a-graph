use crate::common::fresh_copy;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The new exchange goes after the saved ones; the record of another node is not touched.
#[tokio::test]
async fn adds_to_record() {
    let path = "target/test-memory/adds_to_record.json";
    fresh_copy(here(), "files/session.json", path);
    let (p, _) = program(here(), "adds_to_record", Resp::ok(chat("a2", None))).await;
    p.run_node(entity(&p), user("q2")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let contents: Vec<&str> = records[0]["chat"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["content"].as_str().unwrap())
        .collect();
    assert_eq!(contents, ["q1", "a1", "q2", "a2"]);
    assert_eq!(
        (records[1]["id"].as_str(), records[1]["synt"].as_str()),
        (Some("other"), Some("keep me"))
    );
}
