use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the provider wrote when the model of a judge fails stays out of the message: it is the detail.
#[tokio::test]
async fn judge_model_error() {
    let (port, _) = listen(|req| {
        let judging = body_of(req)["format"]["properties"]["validation"].is_object();
        if judging {
            Resp::status(500, r#"{"error":"the patient has diabetes"}"#)
        } else {
            Resp::ok(chat(&user_of(req)))
        }
    })
    .await;
    let p =
        Arc::new(build(&yml_with(here(), "judge_model_error", &[("model", port)])).unwrap_or_else(|e| panic!("{e}")));
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(judge)".to_string())
    );
    assert!(!e.message.contains("diabetes"), "{}", e.message);
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));
}
