use super::*;

/// Check lists what mistral cannot do.
#[tokio::test]
async fn check_limits() {
    let m = client("http://unused");
    assert!(m.check(&ModelRequirements { input_media: vec![MediaKind::Md, MediaKind::Img("png".into())], batch: true, ..Default::default() }).is_ok());

    let bad = ProducedMedia {
        kind: MediaKind::Audio("wav".into()),
        generate: Generate { size: Some(Size { width: 1, height: 1 }), quality: Some("x".into()), background: Some(true), language_code: Some("it".into()), voice: None },
        provider: Some(MediaProvider::Openai(OpenaiMedia { moderation: None, output_compression: None, style: None })),
    };
    let req = ModelRequirements { input_media: vec![MediaKind::Audio("wav".into())], output_media: vec![bad], ..Default::default() };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else { panic!("not Unsupported") };
    let all = problems.join(" | ");
    for needle in ["audio and video input", "size", "quality", "background", "language_code", "provider"] {
        assert!(all.contains(needle), "missing `{needle}` in: {all}");
    }
    let image = ProducedMedia { kind: MediaKind::Img("png".into()), generate: generate(), provider: None };
    assert!(m.check(&ModelRequirements { output_media: vec![image], ..Default::default() }).is_err());
}
