use super::*;

/// Reasoning goes as effort and the thinking switch.
#[tokio::test]
async fn reasoning() {
    for (level, effort) in [(Reasoning::Minimal, "low"), (Reasoning::Low, "low"), (Reasoning::Medium, "medium"), (Reasoning::High, "high"), (Reasoning::Max, "max")] {
        let mut c = call("q");
        c.reasoning = Some(level);
        let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
        let b = body(&req);
        assert_eq!(b["outputConfig"]["effort"], effort);
        assert_eq!(b["additionalModelRequestFields"], json!({ "thinking": { "type": "adaptive", "display": "summarized" } }));
    }
    let mut c = call("q");
    c.reasoning = Some(Reasoning::Off);
    let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
    let b = body(&req);
    assert_eq!(b["additionalModelRequestFields"], json!({ "thinking": { "type": "disabled" } }));
    assert!(b.get("outputConfig").is_none());
}
