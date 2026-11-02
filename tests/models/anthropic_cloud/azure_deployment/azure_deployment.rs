use super::*;

/// Azure names the deployment and takes an api key.
#[tokio::test]
async fn azure_deployment() {
    // SAFETY: a variable only this test uses.
    unsafe { std::env::set_var("A_GRAPH_TEST_AZURE_KEY", "azure-key") };
    let creds = AzureCredentials::ApiKey("A_GRAPH_TEST_AZURE_KEY".into());
    let (base, seen) = serve_once().await;
    let m = Anthropic::azure(&azure_model(creds), &Path::root())
        .unwrap()
        .with_messages_url(format!("{base}/anthropic/v1/messages"));
    let r = m.invoke(&call("hi")).await.unwrap();
    assert_eq!(r.info.provider, "azure_foundry");

    let req = seen.lock().unwrap()[0].clone();
    assert_eq!(req.path, "/anthropic/v1/messages");
    assert!(
        req.headers.contains("api-key: azure-key") && req.headers.contains("anthropic-version: 2023-06-01"),
        "{}",
        req.headers
    );
    let b: Value = serde_json::from_str(&req.body).unwrap();
    assert_eq!(b["model"], "my-claude-deployment");
    assert!(b.get("anthropic_version").is_none());
}
