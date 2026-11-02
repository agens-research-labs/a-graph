use super::*;

/// Reasoning levels.
#[tokio::test]
async fn reasoning_levels() {
    for (level, expected) in [
        (Reasoning::Off, json!(false)),
        (Reasoning::Minimal, json!("low")),
        (Reasoning::Low, json!("low")),
        (Reasoning::Medium, json!("medium")),
        (Reasoning::High, json!("high")),
        (Reasoning::Max, json!("high")),
    ] {
        let mut c = call("q");
        c.reasoning = Some(level);
        let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
        assert_eq!(body(&req)["think"], expected);
    }
}
