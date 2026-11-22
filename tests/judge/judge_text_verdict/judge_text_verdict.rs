use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `structured_output: api`, a verdict that is not one JSON value stops the run, and the message suggests
/// the other mechanism.
#[tokio::test]
async fn judge_text_verdict() {
    let (port, _) = listen(|req: &Req| {
        let judging = body_of(req)["format"]["properties"]["validation"].is_object();
        Resp::ok(chat(&if judging {
            "I approve the text".to_string()
        } else {
            user_of(req)
        }))
    })
    .await;
    let p =
        Arc::new(build(&yml_with(here(), "judge_text_verdict", &[("model", port)])).unwrap_or_else(|e| panic!("{e}")));
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("schema_mismatch", "node(judge)".to_string())
    );
    assert!(e.message.contains("`structured_output: prompt`"), "{}", e.message);
}
