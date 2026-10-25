use super::*;

/// Reasoning levels and the top one.
#[tokio::test]
async fn reasoning_levels() {
    for (level, effort) in [(Reasoning::Off, "none"), (Reasoning::Minimal, "minimal"), (Reasoning::Low, "low"), (Reasoning::Medium, "medium"), (Reasoning::High, "high"), (Reasoning::Max, "xhigh")] {
        let mut c = call("q");
        c.reasoning = Some(level);
        let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
        assert_eq!(body(&req)["reasoning_effort"], effort);
    }
    let (_, req) = one_call(call("q"), Resp::ok(text_answer("x"))).await;
    assert!(body(&req).get("reasoning_effort").is_none());
}
