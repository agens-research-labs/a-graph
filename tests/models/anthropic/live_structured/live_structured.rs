use super::*;

/// Live structured output.
#[tokio::test]
#[ignore = "calls the real API with CLAUDE_API_KEY"]
async fn live_structured() {
    let mut c = call("The capital of Italy.");
    c.max_tokens = Some(100);
    c.schema = Some(
        json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"], "additionalProperties": false }),
    );
    let r = live_model().invoke(&c).await.unwrap();
    // the provider gives the text; a-graph reads the JSON of it
    match a_graph::models::structured::check(c.schema.as_ref().unwrap(), &r) {
        a_graph::models::structured::Checked::Valid(v) => assert_eq!(v["city"], "Rome"),
        other => panic!("{other:?}"),
    }
}
