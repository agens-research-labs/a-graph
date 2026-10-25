use super::*;

/// Azure with ambient credentials sends a bearer token.
#[tokio::test]
async fn azure_ambient() {
    let (base, seen) = serve(|_| Resp::ok(text_answer("hi"))).await;
    let m = OpenAi::azure(&azure_model(AzureCredentials::Ambient, None), &Path::root())
        .unwrap()
        .with_base_url(format!("{base}/openai/v1"))
        .with_token_source(Box::new(StaticToken("entra-token".into())));
    m.invoke(&call("q")).await.unwrap();
    let req = seen.lock().unwrap()[0].clone();
    assert!(req.headers.contains("authorization: bearer entra-token") && !req.headers.contains("api-key"), "{}", req.headers);
}
