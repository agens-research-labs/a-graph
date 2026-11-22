use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Only an answer that is not one JSON value is corrected: a JSON that does not match the schema, a cut answer
/// and a refusal stop the node at the first attempt.
#[tokio::test]
async fn prompt_no_correction() {
    let cut = json!({ "model": "m", "message": { "role": "assistant", "content": "{\"a\":" }, "done": true, "done_reason": "length", "prompt_eval_count": 7, "eval_count": 3 }).to_string();
    for (answer, rule) in [(chat("{\"b\": 1}", None), "schema_mismatch"), (cut, "model_failed")] {
        let (p, seen) = program_seq(here(), "prompt_no_correction", vec![answer, chat("{\"a\": 1}", None)]).await;
        let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
        assert_eq!(e.rule.code(), rule);
        assert_eq!(seen.lock().unwrap().len(), 1, "{rule}: asked again");
    }
}
