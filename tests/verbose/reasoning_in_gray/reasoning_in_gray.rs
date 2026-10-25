use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The reasoning of the model is shown in gray, apart from the answer.
#[tokio::test]
async fn reasoning_in_gray() {
    let thinking = |req: &Req| {
        let text = user_of(req);
        Resp::ok(json!({ "model": "m", "message": { "role": "assistant", "content": text, "thinking": "SECRET THOUGHTS" }, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string())
    };
    let (mut p, _) = program(here(), "reasoning_in_gray", thinking).await;
    let screen = show(&mut p, VerboseLevel::Detail, true, true);
    p.run("hi").await.unwrap();
    let text = screen.text();
    assert!(text.contains("\x1b[90mreasoning: SECRET THOUGHTS"), "{text:?}");
}
