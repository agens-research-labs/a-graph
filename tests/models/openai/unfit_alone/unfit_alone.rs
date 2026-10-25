use super::*;

/// A request that cannot go in a batch fails alone.
#[tokio::test]
async fn unfit_alone() {
    let (handler, _) = batch_server(0, "completed");
    let (base, seen) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut with_media = call("m");
    with_media.output_media = vec![image("png", generate(), None)];
    let results = client(&base).invoke_batch(vec![BatchRequest { id: "one".into(), invocation: call("one") }, BatchRequest { id: "bad".into(), invocation: with_media }], tx).await.unwrap();
    assert!(matches!(results.iter().find(|r| r.id == "bad").unwrap().result, Err(ModelError::Unsupported { .. })));
    assert!(results.iter().find(|r| r.id == "one").unwrap().result.is_ok());
    let upload = seen.lock().unwrap()[0].clone();
    assert_eq!(upload.body.lines().filter(|l| l.starts_with('{')).count(), 1);
}
