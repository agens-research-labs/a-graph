use crate::common::run_node::*;
use crate::common::clean_output;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the memory saves is the question and the answer changed by the utility of the memory, after the one of the output.
#[tokio::test]
async fn memory_write_utility() {
    let path = "target/test-memory/memory_write_utility.json";
    clean_output(path);
    let (p, _) = program(here(), "memory_write_utility", Resp::ok(chat("answer", None))).await;
    p.run_node(entity(&p), user("question")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(records[0]["chat"], json!([{ "role": "user", "content": "[question]" }, { "role": "assistant", "content": "[ANSWER]" }]));
}
