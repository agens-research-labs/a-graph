use super::*;

/// Live a model that is gone comes back as googles own message.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY"]
async fn live_gone_model() {
    let mut c = call("hi");
    c.max_tokens = Some(20);
    let err = live("gemini-2.5-flash").invoke(&c).await.unwrap_err().to_string();
    assert!(err.contains("`flash`") && err.contains("(gemini)") && err.contains("404"), "{err}");
}
