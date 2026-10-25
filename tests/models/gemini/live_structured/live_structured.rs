use super::*;

/// Live structured output.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY"]
async fn live_structured() {
    let mut c = call("The capital of Italy.");
    c.max_tokens = Some(200);
    c.schema = Some(json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }));
    let r = live("gemini-3.5-flash-lite").invoke(&c).await.unwrap();
    assert_eq!(r.structured.unwrap()["city"], "Rome");
}
