use super::*;

/// Live call and thinking.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY"]
async fn live_thinking() {
    let mut c = call("Reply with exactly: ok");
    c.max_tokens = Some(200);
    let r = live("gemini-3.5-flash-lite").invoke(&c).await.unwrap();
    assert!(r.message.unwrap().to_lowercase().contains("ok"));
    assert!(r.info.usage.input_tokens > 0 && r.info.usage.output_tokens > 0);

    let mut c = call("What is 17*23?");
    c.max_tokens = Some(2000);
    c.reasoning = Some(Reasoning::High);
    let r = live("gemini-3.5-flash-lite").invoke(&c).await.unwrap();
    assert!(r.message.unwrap().contains("391"));
    assert!(r.info.usage.reasoning_tokens > 0, "{:?}", r.info.usage);
    assert!(
        r.reasoning.iter().any(|t| t.get("summary").is_some()),
        "no thinking summary came back"
    );
}
