use super::*;

/// Missing ambient credentials are a credentials error at the first call.
#[tokio::test]
async fn ambient_missing() {
    let m = Anthropic::vertex(&vertex_model("global"), &Path::root())
        .unwrap()
        .with_messages_url("http://127.0.0.1:1")
        .with_token_source(Box::new(NoToken));
    let r = m.invoke(&call("hi")).await;
    assert!(
        matches!(&r, Err(ModelError::Credentials { detail, provider, .. }) if detail.contains("no Google credentials") && provider == "vertex_ai"),
        "{r:?}"
    );
}
