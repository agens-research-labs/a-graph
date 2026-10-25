use super::*;

/// A failed job is a provider error and unfit requests fail alone.
#[tokio::test]
async fn failed_job() {
    let (handler, _) = batch_server(0, "Failed", false);
    let (base, _) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = batch_client(&base).invoke_batch(vec![BatchRequest { id: "one".into(), invocation: call("q") }], tx).await;
    assert!(matches!(&r, Err(ModelError::Provider { message, .. }) if message.contains("Failed") && message.contains("could not start")), "{r:?}");

    let (handler, _) = batch_server(0, "Completed", true);
    let (base, seen) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut with_tools = call("t");
    with_tools.tools = vec![ToolDefinition { name: "x".into(), description: "x".into(), parameters: json!({}) }];
    let mut with_schema = call("s");
    with_schema.schema = Some(json!({}));
    let results = batch_client(&base)
        .invoke_batch(
            vec![BatchRequest { id: "one".into(), invocation: call("one") }, BatchRequest { id: "tools".into(), invocation: with_tools }, BatchRequest { id: "schema".into(), invocation: with_schema }],
            tx,
        )
        .await
        .unwrap();
    for id in ["tools", "schema"] {
        assert!(matches!(results.iter().find(|r| r.id == id).unwrap().result, Err(ModelError::Unsupported { .. })), "{id}");
    }
    assert!(results.iter().find(|r| r.id == "one").unwrap().result.is_ok());
    let put = seen.lock().unwrap().iter().find(|r| r.method == "PUT").cloned().unwrap();
    assert_eq!(put.body.lines().count(), 1);
}
