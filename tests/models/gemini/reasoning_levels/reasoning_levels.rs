use super::*;

/// Reasoning levels.
#[tokio::test]
async fn reasoning_levels() {
    for (level, expected) in [(Reasoning::Minimal, "minimal"), (Reasoning::Low, "low"), (Reasoning::Medium, "medium"), (Reasoning::High, "high"), (Reasoning::Max, "high")] {
        let mut c = call("q");
        c.reasoning = Some(level);
        let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
        assert_eq!(body(&req)["generation_config"], json!({ "thinking_level": expected, "thinking_summaries": "auto" }));
    }
    // absent: nothing is sent
    let (_, req) = one_call(call("q"), Resp::ok(text_answer("x"))).await;
    assert!(body(&req).get("generation_config").is_none());

    // Gemini has no way to switch thinking off: refused before any call
    let mut c = call("q");
    c.reasoning = Some(Reasoning::Off);
    let r = client("http://127.0.0.1:1").invoke(&c).await;
    assert!(matches!(r, Err(ModelError::Unsupported { .. })), "{r:?}");
}
