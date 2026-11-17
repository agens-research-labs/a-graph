use super::*;

/// Asked, the logprobs are included with their alternatives, and the tokens of every text part come
/// back in one list, in order.
#[tokio::test]
async fn logprobs_asked() {
    let mut c = call("q");
    c.logprobs = Some(2);
    let token = |t: &str, lp: f64| json!({ "token": t, "logprob": lp, "bytes": t.as_bytes(), "top_logprobs": [{ "token": "x", "logprob": -3.5, "bytes": [120] }] });
    let output = json!([{ "id": "msg_1", "type": "message", "role": "assistant", "status": "completed", "content": [
        { "type": "output_text", "text": "Ro", "annotations": [], "logprobs": [token("Ro", -0.1)] },
        { "type": "output_text", "text": "ma", "annotations": [], "logprobs": [token("ma", -0.2)] }
    ] }]);
    let (r, req) = one_call(c, Resp::ok(response("completed", output))).await;
    let r = r.unwrap();

    let b = body(&req);
    assert_eq!(b["include"], json!(["message.output_text.logprobs"]));
    assert_eq!(b["top_logprobs"], json!(2));
    assert_eq!(r.message.as_deref(), Some("Roma"));
    let tokens: Vec<(&str, f64)> = r.logprobs[0].iter().map(|t| (t.token.as_str(), t.logprob)).collect();
    assert_eq!((r.logprobs.len(), tokens), (1, vec![("Ro", -0.1), ("ma", -0.2)]));
    assert_eq!(r.logprobs[0][0].bytes, Some(b"Ro".to_vec()));
    assert_eq!(
        r.logprobs[0][0].top_logprobs,
        vec![TopLogprob {
            token: "x".into(),
            logprob: -3.5,
            bytes: Some(vec![120]),
        }]
    );
}
