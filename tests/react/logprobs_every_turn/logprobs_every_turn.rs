use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The controller gives the tokens of all its turns, one list for each turn, in order: those of the
/// whole routing output, not only of its `answer`.
#[tokio::test]
async fn logprobs_every_turn() {
    let turns = AtomicUsize::new(0);
    let model = move |req: &Req| {
        if !is_controller(req) {
            return Resp::ok(chat(&user_of(req)));
        }
        let n = turns.fetch_add(1, Ordering::SeqCst);
        let routing = if n == 0 {
            json!({ "done": false, "next_worker": "w1", "input": "find X", "reasoning": "r" })
        } else {
            json!({ "done": true, "answer": "X" })
        };
        let token = format!("turn{n}");
        Resp::ok(json!({ "model": "m", "message": { "role": "assistant", "content": routing.to_string() }, "done": true, "done_reason": "stop",
            "logprobs": [{ "token": token, "logprob": -0.1 }], "prompt_eval_count": 1, "eval_count": 1 }).to_string())
    };
    let (p, _) = program(here(), "logprobs_every_turn", model).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(text(&out, "c"), "X");
    let c = out.items.iter().find(|i| i.agent_id == "c").unwrap();
    let calls: Vec<Vec<&str>> = c
        .logprobs
        .iter()
        .map(|l| l.iter().map(|t| t.token.as_str()).collect())
        .collect();
    assert_eq!(calls, vec![vec!["turn0"], vec!["turn1"]]);
    let w1 = out.items.iter().find(|i| i.agent_id == "w1").unwrap();
    assert!(w1.logprobs.is_empty(), "a worker that does not ask has none");
}
