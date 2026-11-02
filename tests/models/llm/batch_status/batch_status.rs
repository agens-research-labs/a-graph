use super::*;

/// A batch reports its status and matches results by id.
#[tokio::test]
async fn batch_status() {
    let model = Fake { id: "fast" };
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let requests = vec![
        BatchRequest {
            id: "one".into(),
            invocation: invocation("s", "first"),
        },
        BatchRequest {
            id: "two".into(),
            invocation: invocation("s", "second"),
        },
    ];
    let results = model.invoke_batch(requests, tx).await.unwrap();

    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    assert_eq!(
        events.first(),
        Some(&BatchEvent::Submitted {
            batch_id: "b1".into(),
            requests: 2
        })
    );
    assert_eq!(
        events.iter().filter(|e| matches!(e, BatchEvent::Status { .. })).count(),
        2
    );
    assert_eq!(
        events.last(),
        Some(&BatchEvent::Finished {
            batch_id: "b1".into(),
            outcome: BatchOutcome::Completed
        })
    );

    let by_id = |id: &str| {
        results
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .message
            .clone()
    };
    assert_eq!(by_id("one").as_deref(), Some("first"));
    assert_eq!(by_id("two").as_deref(), Some("second"));
}
