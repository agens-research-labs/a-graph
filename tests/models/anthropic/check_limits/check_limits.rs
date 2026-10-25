use super::*;

/// Check lists what anthropic cannot do.
#[tokio::test]
async fn check_limits() {
    let m = client("http://unused");
    assert!(m.check(&ModelRequirements { max_tokens: Some(10), input_media: vec![MediaKind::Md, MediaKind::Img("png".into())], ..Default::default() }).is_ok());

    let req = ModelRequirements {
        max_tokens: None,
        input_media: vec![MediaKind::Audio("wav".into()), MediaKind::Img("bmp".into())],
        output_media: vec![ProducedMedia { kind: MediaKind::Img("png".into()), generate: serde_norway_generate(), provider: None }],
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else { panic!("not Unsupported") };
    assert_eq!(problems.len(), 4, "{problems:?}");
}
