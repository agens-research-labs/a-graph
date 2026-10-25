use super::*;

/// Azure uses the same api with api key and its own endpoint.
#[tokio::test]
async fn azure_api_key() {
    let at = Path::root();
    assert_eq!(OpenAi::azure(&azure_model(azure_key(), None), &at).unwrap().endpoint(), "https://my-res.openai.azure.com/openai/v1");

    let (base, seen) = serve(|_| Resp::ok(text_answer("hi"))).await;
    let m = OpenAi::azure(&azure_model(azure_key(), None), &at).unwrap().with_base_url(format!("{base}/openai/v1"));
    let r = m.invoke(&call("q")).await.unwrap();
    assert_eq!(r.info.provider, "azure_foundry");
    let req = seen.lock().unwrap()[0].clone();
    assert_eq!(req.path, "/openai/v1/responses");
    assert!(req.headers.contains("api-key: azure-key") && !req.headers.contains("authorization"), "{}", req.headers);
    // the model of the yml is the name of the deployment
    assert_eq!(body(&req)["model"], "my-gpt-deployment");
}
