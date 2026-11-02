use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The memory saves what the node used and delivered: the question changed on the way in and the answer changed on the way out.
#[tokio::test]
async fn memory_gets_changed() {
    let path = "target/test-memory/memory_gets_changed.json";
    clean_output(path);
    let (p, _) = program(here(), "memory_gets_changed", Resp::ok(chat("answer", None))).await;
    p.run_node(entity(&p), user("question")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        records[0]["chat"],
        json!([{ "role": "user", "content": "QUESTION" }, { "role": "assistant", "content": "[answer]" }])
    );
}
