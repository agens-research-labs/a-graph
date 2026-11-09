use super::*;

/// Custom is the same contract under its own name.
#[tokio::test]
async fn custom_provider() {
    let (base, _) = serve(|_| Resp::status(500, json!({ "error": "boom" }).to_string())).await;
    let m = Ollama::custom(&local(&format!("{base}/")), &Path::root()).unwrap();
    let e = m.invoke(&call("q")).await.unwrap_err();
    let text = e.to_string();
    assert!(text.contains("(custom)") && !text.contains("boom"), "{text}");
    assert_eq!(e.said(), Some("boom"));
}
