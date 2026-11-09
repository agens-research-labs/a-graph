use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A tool call in an answer cut at `max_tokens` is not run: the run stops.
#[tokio::test]
async fn cut_call_not_run() {
    let (tool_port, service) = listen(|_| Resp::ok(json!({ "temp": 22 }).to_string())).await;
    // the model always asks for the tool, and its answer stops at max_tokens
    let cut = |_: &Req| {
        Resp::ok(
            json!({ "model": "m", "done": true, "done_reason": "length", "prompt_eval_count": 1, "eval_count": 1,
                    "message": { "role": "assistant", "content": "",
                                 "tool_calls": [{ "function": { "name": "t", "arguments": { "city": "Ro" } } }] } })
            .to_string(),
        )
    };
    let (p, seen) = program_with(here(), "cut_call_not_run", &[("tool", tool_port)], cut).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(n)".to_string())
    );
    assert!(e.message.contains("cut before its end"), "{}", e.message);
    assert!(service.lock().unwrap().is_empty(), "the tool was called");
    assert_eq!(
        seen.lock().unwrap().len(),
        1,
        "one call of the model, then the run stops"
    );
}
