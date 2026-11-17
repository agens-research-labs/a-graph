use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An output utility changes the text, not the logprobs: they stay those of the answer of the model.
#[tokio::test]
async fn logprobs_before_util() {
    let answer = json!({ "model": "m", "message": { "role": "assistant", "content": "answer" }, "done": true, "done_reason": "stop",
        "logprobs": [{ "token": "answer", "logprob": -0.5 }], "prompt_eval_count": 1, "eval_count": 1 });
    let (p, _) = program(here(), "logprobs_before_util", Resp::ok(answer.to_string())).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.assistant.message, Some(Content::Text("[answer]".into())));
    let tokens: Vec<&str> = out.assistant.logprobs[0].iter().map(|t| t.token.as_str()).collect();
    assert_eq!(tokens, vec!["answer"]);
}
