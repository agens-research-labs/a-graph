use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The body of the POST is the input as JSON, the headers come from the variables, and the answer is read.
#[tokio::test]
async fn http_roundtrip() {
    // SAFETY: the name of the variable the yml writes; it is never used to call anything.
    unsafe { std::env::set_var("A_GRAPH_TEST_UTIL_HTTP", "secret") };
    let (utility_port, seen) = listen(|_| Resp::ok(r#"{"text":"converted"}"#)).await;
    let u = manager(&yml_with(here(), "http_roundtrip", &[("utility", utility_port)]))
        .get("u")
        .unwrap()
        .clone();
    let out = call_utility(
        &u,
        &UtilityInput::File {
            path: "a.pdf".into(),
            mime_type: Some("application/pdf".into()),
        },
    )
    .await
    .unwrap();
    assert_eq!(out, UtilityOutput::Text("converted".into()));
    let seen = seen.lock().unwrap();
    assert_eq!((seen[0].method.as_str(), seen[0].path.as_str()), ("POST", "/convert"));
    assert!(seen[0].headers.contains("authorization: secret"), "{}", seen[0].headers);
    let body: Value = serde_json::from_str(&seen[0].body).unwrap();
    assert_eq!(body, json!({ "path": "a.pdf", "mime_type": "application/pdf" }));
}
