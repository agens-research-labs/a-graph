use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the provider wrote when the model of a controller fails stays out of the message: it is the
/// detail.
#[tokio::test]
async fn controller_error() {
    let (p, _) = program(here(), "controller_error", |req| {
        if is_controller(req) {
            Resp::status(500, r#"{"error":"the patient has diabetes"}"#)
        } else {
            Resp::ok(chat(&user_of(req)))
        }
    })
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "model_failed");
    assert!(!e.message.contains("diabetes"), "{}", e.message);
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));
}
