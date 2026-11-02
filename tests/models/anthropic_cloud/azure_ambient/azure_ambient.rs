use super::*;

/// Azure with ambient credentials sends a bearer token.
#[tokio::test]
async fn azure_ambient() {
    let (base, seen) = serve_once().await;
    let m = Anthropic::azure(&azure_model(AzureCredentials::Ambient), &Path::root())
        .unwrap()
        .with_messages_url(format!("{base}/anthropic/v1/messages"))
        .with_token_source(Box::new(StaticToken("entra-token".into())));
    m.invoke(&call("hi")).await.unwrap();
    let req = seen.lock().unwrap()[0].clone();
    assert!(
        req.headers.contains("authorization: bearer entra-token")
            && req.headers.contains("anthropic-version: 2023-06-01"),
        "{}",
        req.headers
    );
}
