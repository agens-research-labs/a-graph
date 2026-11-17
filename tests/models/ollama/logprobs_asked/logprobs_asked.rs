use super::*;

/// Asked, the logprobs are switched on with their alternatives, and those of the answer come back;
/// not asked, nothing is sent.
#[tokio::test]
async fn logprobs_asked() {
    let mut c = call("q");
    c.logprobs = Some(4);
    let answer = json!({ "model": "llama3.2:latest", "message": { "role": "assistant", "content": "ok" }, "done": true, "done_reason": "stop",
        "logprobs": [{ "token": "ok", "logprob": -0.01, "bytes": [111, 107], "top_logprobs": [{ "token": "no", "logprob": -5.0, "bytes": [110, 111] }] }],
        "prompt_eval_count": 3, "eval_count": 1 })
    .to_string();
    let (r, req) = one_call(c, Resp::ok(answer)).await;
    let r = r.unwrap();
    let b = body(&req);
    assert_eq!(
        (b["logprobs"].clone(), b["top_logprobs"].clone()),
        (json!(true), json!(4))
    );
    assert_eq!(
        r.logprobs,
        vec![vec![TokenLogprob {
            token: "ok".into(),
            logprob: -0.01,
            bytes: Some(b"ok".to_vec()),
            top_logprobs: vec![TopLogprob {
                token: "no".into(),
                logprob: -5.0,
                bytes: Some(b"no".to_vec()),
            }],
        }]]
    );

    let (_, req) = one_call(call("q"), Resp::ok(text_answer("x"))).await;
    let b = body(&req);
    assert!(b.get("logprobs").is_none() && b.get("top_logprobs").is_none(), "{b}");
}
