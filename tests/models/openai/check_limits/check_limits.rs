use super::*;

/// Check lists what openai cannot do.
#[tokio::test]
async fn check_limits() {
    let m = client("http://unused");
    assert!(m.check(&ModelRequirements { input_media: vec![MediaKind::Md, MediaKind::Img("png".into())], batch: true, ..Default::default() }).is_ok());

    let bad_audio = ProducedMedia {
        kind: MediaKind::Audio("wav".into()),
        generate: Generate { size: Some(Size { width: 1, height: 1 }), quality: Some("x".into()), background: Some(true), language_code: Some("it".into()), voice: None },
        provider: Some(MediaProvider::Openai(OpenaiMedia { moderation: None, output_compression: None, style: None })),
    };
    let req = ModelRequirements { input_media: vec![MediaKind::Video("mp4".into())], output_media: vec![bad_audio], ..Default::default() };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else { panic!("not Unsupported") };
    let all = problems.join(" | ");
    for needle in ["audio and video input", "language_code", "voice", "size", "quality", "background", "provider"] {
        assert!(all.contains(needle), "missing `{needle}` in: {all}");
    }
    let two = ModelRequirements { output_media: vec![image("png", generate(), None), image("png", generate(), None)], ..Default::default() };
    assert!(m.check(&two).is_err());
    let gemini_fields = image("png", generate(), Some(MediaProvider::Gemini(GeminiMedia { image_size: None, aspect_ratio: None })));
    assert!(m.check(&ModelRequirements { output_media: vec![gemini_fields], ..Default::default() }).is_err());
}
