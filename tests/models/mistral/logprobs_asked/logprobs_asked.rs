use super::*;

/// Asked, the logprobs are switched on with their alternatives, and those of the choice come back.
#[tokio::test]
async fn logprobs_asked() {
    let mut c = call("q");
    c.logprobs = Some(3);
    let answer = json!({ "id": "c1", "object": "chat.completion", "model": "mistral-small-2506", "created": 1,
        "choices": [{ "index": 0, "message": { "role": "assistant", "content": "Roma" }, "finish_reason": "stop",
            "logprobs": { "content": [{ "token": "Roma", "logprob": -0.3, "bytes": [82, 111, 109, 97], "token_id": 9,
                "top_logprobs": [{ "token": "Milano", "logprob": -2.0, "bytes": [77], "token_id": 10 }] }] } }],
        "usage": { "prompt_tokens": 3, "completion_tokens": 1, "total_tokens": 4 } })
    .to_string();
    let (r, req) = one_call(c, Resp::ok(answer)).await;
    let r = r.unwrap();
    let b = body(&req);
    assert_eq!(
        (b["logprobs"].clone(), b["top_logprobs"].clone()),
        (json!(true), json!(3))
    );
    assert_eq!(
        r.logprobs,
        vec![vec![TokenLogprob {
            token: "Roma".into(),
            logprob: -0.3,
            bytes: Some(b"Roma".to_vec()),
            top_logprobs: vec![TopLogprob {
                token: "Milano".into(),
                logprob: -2.0,
                bytes: Some(vec![77]),
            }],
        }]]
    );

    // not asked, nothing is sent
    let (_, req) = one_call(call("q"), Resp::ok(text_answer("x"))).await;
    let b = body(&req);
    assert!(b.get("logprobs").is_none() && b.get("top_logprobs").is_none(), "{b}");
}
