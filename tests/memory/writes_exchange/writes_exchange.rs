use crate::common::run_node::*;
use crate::common::clean_output;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The question and the answer of the turn are saved; the file and the record are created.
#[tokio::test]
async fn writes_exchange() {
    let path = "target/test-memory/writes_exchange.json";
    clean_output(path);
    let (p, _) = program(here(), "writes_exchange", Resp::ok(chat("the answer", None))).await;
    p.run_node(entity(&p), user("the question")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(records.as_array().unwrap().len(), 1);
    let r = &records[0];
    assert_eq!((r["id"].as_str(), r["synt"].as_str()), (Some("n"), Some("")));
    assert_eq!(r["chat"], json!([{ "role": "user", "content": "the question" }, { "role": "assistant", "content": "the answer" }]));
    let ts = r["timestamp"].as_str().unwrap();
    assert!(chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d_%H:%M:%S").is_ok(), "{ts}");
}
