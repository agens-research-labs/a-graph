use super::*;

/// Live structured output.
#[tokio::test]
#[ignore = "calls the real API with CLAUDE_API_KEY"]
async fn live_structured() {
    let mut c = call("The capital of Italy.");
    c.max_tokens = Some(100);
    c.schema = Some(json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"], "additionalProperties": false }));
    let r = live_model().invoke(&c).await.unwrap();
    assert_eq!(r.structured.unwrap()["city"], "Rome");
}
