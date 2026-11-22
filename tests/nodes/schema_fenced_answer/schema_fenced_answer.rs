use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A JSON in one code fence is the JSON alone; two values, or text around the JSON, are not usable JSON.
#[tokio::test]
async fn schema_fenced_answer() {
    for fenced in [
        "```json\n{\"a\": 1}\n```",
        "```\n{\"a\": 1}\n```",
        "  ```json\n{\"a\": 1}\n```\n",
    ] {
        let (p, _) = program(here(), "schema_fenced_answer", Resp::ok(chat(fenced, None))).await;
        let out = p.run_node(entity(&p), user("q")).await.unwrap();
        assert_eq!(
            out.assistant.message,
            Some(Content::Json(json!({ "a": 1 }))),
            "{fenced}"
        );
    }
    for refused in [
        "{\"a\": 1} {\"a\": 2}",
        "Here it is: {\"a\": 1}",
        "```json\n{\"a\": 1}\n```\nDone.",
        "```json\n{\"a\": 1}\n```\n```json\n{\"a\": 2}\n```",
    ] {
        let (p, _) = program(here(), "schema_fenced_answer", Resp::ok(chat(refused, None))).await;
        let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
        assert_eq!(e.rule.code(), "schema_mismatch", "{refused}");
    }
}
