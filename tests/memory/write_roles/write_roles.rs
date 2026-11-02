use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A role the node does not write is not saved.
#[tokio::test]
async fn write_roles() {
    let path = "target/test-memory/write_roles.json";
    clean_output(path);
    let (p, _) = program(here(), "write_roles", Resp::ok(chat("the answer", None))).await;
    p.run_node(entity(&p), user("the question")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        records[0]["chat"],
        json!([{ "role": "user", "content": "the question" }])
    );
}
