use super::*;

/// A token without its `logprob`, or with `bytes` that are not bytes, is a wrong answer, not a
/// token dropped in silence.
#[tokio::test]
async fn logprobs_malformed() {
    for bad in [
        json!([{ "token": "ok" }]),
        json!([{ "token": "ok", "logprob": -0.1, "bytes": [300] }]),
    ] {
        let mut c = call("q");
        c.logprobs = Some(0);
        let answer = json!({ "model": "m", "message": { "role": "assistant", "content": "ok" }, "done": true, "done_reason": "stop", "logprobs": bad })
            .to_string();
        let (r, _) = one_call(c, Resp::ok(answer)).await;
        let e = r.unwrap_err();
        assert!(matches!(e, ModelError::InvalidResponse { .. }), "{e:?}");
    }
}
