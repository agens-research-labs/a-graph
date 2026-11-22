use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `schema`, a JSON cut at `max_tokens` stops the node and asks to increase `max_tokens`.
#[tokio::test]
async fn schema_cut_answer() {
    let cut = json!({ "model": "m", "message": { "role": "assistant", "content": "{\"a\":" }, "done": true, "done_reason": "length", "prompt_eval_count": 7, "eval_count": 3 });
    let (p, _) = program(here(), "schema_cut_answer", Resp::ok(cut.to_string())).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "model_failed");
    assert!(e.message.contains("increase `max_tokens`"), "{}", e.message);
}
