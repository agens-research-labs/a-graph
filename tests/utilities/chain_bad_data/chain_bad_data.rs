use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file that is not valid base64 never reaches a utility: the node stops, with the reason as the detail.
#[tokio::test]
async fn chain_bad_data() {
    let (utility, got) = listen(|r| match r.path.as_str() {
        "/first" => Resp::ok(r#"{"mime_type":"image/png","data":"not base64!"}"#),
        _ => Resp::ok(r#"{"text":"converted"}"#),
    })
    .await;
    let (p, seen) = program_with(
        here(),
        "chain_bad_data",
        &[("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let e = p.run_node(entity(&p), user("what is it")).await.unwrap_err();
    assert_eq!(e.rule.code(), "utility_failed");
    assert!(
        e.message.contains("utility `second`")
            && e.message.contains("not valid base64")
            && !e.message.contains("Invalid symbol"),
        "{}",
        e.message
    );
    assert!(
        e.detail.as_deref().is_some_and(|d| d.contains("Invalid symbol")),
        "{:?}",
        e.detail
    );
    let got = got.lock().unwrap();
    assert_eq!(got.len(), 1, "the second utility is never called");
    assert!(seen.lock().unwrap().is_empty(), "no call to the model");
}
