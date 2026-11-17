use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// In the result written for a program, every item has `logprobs`: one list for each call, every
/// token with its bytes and alternatives; empty for a node that does not ask them.
#[tokio::test]
async fn logprobs_in_json() {
    let answer = json!({ "model": "m", "message": { "role": "assistant", "content": "ok" }, "done": true, "done_reason": "stop",
        "logprobs": [{ "token": "ok", "logprob": -0.5, "bytes": [111, 107], "top_logprobs": [{ "token": "no", "logprob": -2.5 }] }],
        "prompt_eval_count": 1, "eval_count": 1 });
    let (p, _) = program(here(), "logprobs_in_json", Resp::ok(answer.to_string())).await;
    let out = Arc::new(p).run("q").await.unwrap().to_json();
    assert_eq!(
        out["items"][0]["logprobs"],
        json!([[{ "token": "ok", "logprob": -0.5, "bytes": [111, 107], "top_logprobs": [{ "token": "no", "logprob": -2.5, "bytes": null }] }]])
    );
    assert_eq!(
        (out["items"][1]["node"].clone(), out["items"][1]["logprobs"].clone()),
        (json!("b"), json!([]))
    );
}
