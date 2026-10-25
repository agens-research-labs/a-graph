use super::*;

/// Check and batch.
#[tokio::test]
async fn check_batch() {
    let m = Ollama::ollama(&local("http://unused"), &Path::root()).unwrap();
    assert!(m.check(&ModelRequirements { input_media: vec![MediaKind::Md, MediaKind::Img("png".into())], reasoning: Some(Reasoning::Off), ..Default::default() }).is_ok());

    let req = ModelRequirements {
        batch: true,
        input_media: vec![MediaKind::Video("mp4".into())],
        output_media: vec![ProducedMedia { kind: MediaKind::Img("png".into()), generate: Generate { voice: None, language_code: None, size: None, quality: None, background: None }, provider: None }],
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else { panic!("not Unsupported") };
    assert_eq!(problems.len(), 3, "{problems:?}");

    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = m.invoke_batch(vec![BatchRequest { id: "one".into(), invocation: call("q") }], tx).await;
    assert!(matches!(r, Err(ModelError::Unsupported { .. })));
}
