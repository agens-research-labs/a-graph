use super::*;

/// Reasoning levels and off.
#[tokio::test]
async fn reasoning_levels() {
    for (level, effort) in [(Reasoning::Low, "low"), (Reasoning::Medium, "medium"), (Reasoning::High, "high"), (Reasoning::Max, "max")] {
        let mut c = call("q");
        c.reasoning = Some(level);
        let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
        assert_eq!(body(&req)["output_config"]["effort"], effort);
    }
    let mut c = call("q");
    c.reasoning = Some(Reasoning::Off);
    let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
    let b = body(&req);
    assert_eq!(b["thinking"], json!({ "type": "disabled" }));
    assert!(b.get("output_config").is_none());

    // absent: nothing is sent
    let (_, req) = one_call(call("q"), Resp::ok(text_answer("x"))).await;
    assert!(body(&req).get("thinking").is_none());
}
