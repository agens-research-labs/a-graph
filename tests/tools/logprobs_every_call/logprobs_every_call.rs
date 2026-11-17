use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The tokens of every call of the tool loop reach the final output, one list for each call, in
/// order: the last list is the final answer.
#[tokio::test]
async fn logprobs_every_call() {
    let model = |req: &Req| {
        let lp = |t: &str| json!([{ "token": t, "logprob": -1.0 }]);
        if tool_messages(req).is_empty() {
            let asked = json!([{ "function": { "name": "t", "arguments": { "city": "Roma" } } }]);
            Resp::ok(json!({ "model": "m", "message": { "role": "assistant", "content": "", "tool_calls": asked }, "done": true, "done_reason": "stop", "logprobs": lp("call"), "prompt_eval_count": 1, "eval_count": 1 }).to_string())
        } else {
            Resp::ok(json!({ "model": "m", "message": { "role": "assistant", "content": "done" }, "done": true, "done_reason": "stop", "logprobs": lp("done"), "prompt_eval_count": 1, "eval_count": 1 }).to_string())
        }
    };
    let (p, seen) = program(here(), "logprobs_every_call", model).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(seen.lock().unwrap().len(), 2, "two calls");
    let calls: Vec<Vec<&str>> = out.items[0]
        .logprobs
        .iter()
        .map(|c| c.iter().map(|t| t.token.as_str()).collect())
        .collect();
    assert_eq!(calls, vec![vec!["call"], vec!["done"]]);
}
