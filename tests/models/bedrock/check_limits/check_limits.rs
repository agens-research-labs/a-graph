use super::*;

/// Check lists what bedrock cannot do.
#[tokio::test]
async fn check_limits() {
    let m = client("http://unused");
    assert!(
        m.check(&ModelRequirements {
            input_media: vec![MediaKind::Md, MediaKind::Img("png".into())],
            ..Default::default()
        })
        .is_ok()
    );
    let req = ModelRequirements {
        batch: true,
        uses_tools: true,
        schema: Some(json!({})),
        input_media: vec![MediaKind::Audio("wav".into()), MediaKind::Img("bmp".into())],
        output_media: vec![ProducedMedia {
            kind: MediaKind::Img("png".into()),
            generate: Generate::default(),
            provider: None,
        }],
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else {
        panic!("not Unsupported")
    };
    // media output, tools/schema in a batch, no `batch` block, audio input, a bmp image
    assert_eq!(problems.len(), 5, "{problems:?}");
}
