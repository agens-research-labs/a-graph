use super::*;

/// A node calls a model only by id.
#[tokio::test]
async fn call_by_id() {
    let mut manager = ModelManager::new();
    assert!(manager.insert("fast", Box::new(Fake { id: "fast" })).is_none());
    assert!(manager.insert("strong", Box::new(Fake { id: "strong" })).is_none());
    assert_eq!(manager.len(), 2);
    assert!(manager.get("ghost").is_none());

    let r = manager
        .get("strong")
        .unwrap()
        .invoke(&invocation("be brief", "hi"))
        .await
        .unwrap();
    assert_eq!(r.message.as_deref(), Some("be brief|hi"));
    assert_eq!(r.info.model, "strong");
    assert_eq!(r.stop_reason, StopReason::End);
}
