use crate::common::run_node::*;
use a_graph::models::TokenLogprob;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The logprobs are asked as the node says, and go to the final output, never to the next node.
#[tokio::test]
async fn logprobs_to_output() {
    let answer = json!({ "model": "m", "message": { "role": "assistant", "content": "ok" }, "done": true, "done_reason": "stop",
        "logprobs": [{ "token": "ok", "logprob": -0.25, "bytes": [111, 107], "top_logprobs": [] }], "prompt_eval_count": 1, "eval_count": 1 });
    let (p, seen) = program(here(), "logprobs_to_output", Resp::ok(answer.to_string())).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    let body = sent(&seen);
    assert_eq!(
        (body["logprobs"].clone(), body["top_logprobs"].clone()),
        (json!(true), json!(2))
    );
    assert_eq!(
        out.assistant.logprobs,
        vec![vec![TokenLogprob {
            token: "ok".into(),
            logprob: -0.25,
            bytes: Some(b"ok".to_vec()),
            top_logprobs: vec![],
        }]]
    );
    assert_eq!(out.node.message, Some(Content::Text("ok".into())));
    assert!(out.node.logprobs.is_empty(), "nothing goes to the connected node");
}
