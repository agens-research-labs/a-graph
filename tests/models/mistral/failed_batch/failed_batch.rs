use super::*;

/// A failed batch is a provider error and a media request fails alone.
#[tokio::test]
async fn failed_batch() {
    let (handler, _) = batch_server(0, "FAILED");
    let (base, _) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = client(&base)
        .invoke_batch(
            vec![BatchRequest {
                id: "one".into(),
                invocation: call("q"),
            }],
            tx,
        )
        .await;
    assert!(
        matches!(&r, Err(ModelError::Provider { message, .. }) if message.contains("FAILED") && message.contains("the job was invalid")),
        "{r:?}"
    );

    let (handler, _) = batch_server(0, "SUCCESS");
    let (base, seen) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut with_media = call("m");
    with_media.output_media = vec![ProducedMedia {
        kind: MediaKind::Audio("mp3".into()),
        generate: generate(),
        provider: None,
    }];
    let results = client(&base)
        .invoke_batch(
            vec![
                BatchRequest {
                    id: "one".into(),
                    invocation: call("one"),
                },
                BatchRequest {
                    id: "bad".into(),
                    invocation: with_media,
                },
            ],
            tx,
        )
        .await
        .unwrap();
    assert!(matches!(
        results.iter().find(|r| r.id == "bad").unwrap().result,
        Err(ModelError::Unsupported { .. })
    ));
    assert!(results.iter().find(|r| r.id == "one").unwrap().result.is_ok());
    assert_eq!(body(&seen.lock().unwrap()[0])["requests"].as_array().unwrap().len(), 1);
}
