use super::*;

/// With tools the logprobs join the encrypted reasoning in `include`; with 0 alternatives no
/// `top_logprobs` is sent.
#[tokio::test]
async fn logprobs_with_tools() {
    let mut c = call("q");
    c.logprobs = Some(0);
    c.tools = vec![ToolDefinition {
        name: "t".into(),
        description: "d".into(),
        parameters: json!({ "type": "object" }),
    }];
    let (r, req) = one_call(c, Resp::ok(text_answer("hello"))).await;
    r.unwrap();
    let b = body(&req);
    assert_eq!(
        b["include"],
        json!(["reasoning.encrypted_content", "message.output_text.logprobs"])
    );
    assert!(b.get("top_logprobs").is_none(), "{b}");
}
