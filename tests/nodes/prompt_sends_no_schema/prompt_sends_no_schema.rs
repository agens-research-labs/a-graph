use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `schema_prompt` the provider gets no schema: the schema is written, as it is, at the end of the system
/// prompt, and a-graph reads the JSON of the answer.
#[tokio::test]
async fn prompt_sends_no_schema() {
    let (p, seen) = program(here(), "prompt_sends_no_schema", Resp::ok(chat("{\"a\": 1}", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.assistant.message, Some(Content::Json(json!({ "a": 1 }))));
    let body = sent(&seen);
    assert!(body.get("format").is_none(), "{body}");
    let system = body["messages"][0].clone();
    assert_eq!(system["role"], "system", "{body}");
    let text = system["content"].as_str().unwrap();
    assert!(text.starts_with("Answer with one JSON value only"), "{text}");
    let schema = text.split_once("JSON Schema:\n").unwrap().1;
    assert_eq!(
        serde_json::from_str::<Value>(schema).unwrap(),
        json!({ "type": "object", "properties": { "a": { "type": "integer" } }, "required": ["a"] })
    );
}
