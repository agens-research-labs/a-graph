use super::*;

/// Reasoning levels map one to one.
#[tokio::test]
async fn reasoning_levels() {
    for (level, effort) in [
        (Reasoning::Off, "none"),
        (Reasoning::Minimal, "minimal"),
        (Reasoning::Low, "low"),
        (Reasoning::Medium, "medium"),
        (Reasoning::High, "high"),
        (Reasoning::Max, "max"),
    ] {
        let mut c = call("q");
        c.reasoning = Some(level);
        let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
        let b = body(&req);
        assert_eq!(b["reasoning"]["effort"], effort);
        // the summary is asked whenever there is reasoning; the encrypted items come back for the tool loop
        assert_eq!(b["reasoning"].get("summary").is_some(), level != Reasoning::Off);
        assert_eq!(b["include"], json!(["reasoning.encrypted_content"]));
    }
}
