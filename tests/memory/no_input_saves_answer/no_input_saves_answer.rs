use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that takes no message saves only its answer: the message of the user is not its own.
#[tokio::test]
async fn no_input_saves_answer() {
    let path = "target/test-memory/no_input_saves_answer.json";
    clean_output(path);
    let (p, _) = program(here(), "no_input_saves_answer", Resp::ok(chat("the answer", None))).await;
    p.run_node(entity(&p), user("the text of the user")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        records[0]["chat"],
        json!([{ "role": "assistant", "content": "the answer" }])
    );
}
