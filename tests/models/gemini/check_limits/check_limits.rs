use super::*;

/// Check lists what gemini cannot do.
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

    let bad_image = ProducedMedia {
        kind: MediaKind::Img("png".into()),
        generate: Generate {
            size: Some(Size {
                width: 1024,
                height: 1024,
            }),
            quality: Some("high".into()),
            background: Some(true),
            language_code: Some("it-IT".into()),
            voice: Some("x".into()),
        },
        provider: Some(MediaProvider::Openai(OpenaiMedia {
            moderation: None,
            output_compression: None,
            style: None,
        })),
    };
    let req = ModelRequirements {
        reasoning: Some(Reasoning::Off),
        batch: true,
        input_media: vec![MediaKind::Audio("wav".into())],
        output_media: vec![bad_image],
        schema: Some(json!({})),
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else {
        panic!("not Unsupported")
    };
    let all = problems.join(" | ");
    for needle in [
        "switch thinking off",
        "batch",
        "audio and video input",
        "size",
        "quality",
        "background",
        "language_code",
        "voice",
        "openai",
        "schema and a produced media",
    ] {
        assert!(all.contains(needle), "missing `{needle}` in: {all}");
    }

    let two = ModelRequirements {
        output_media: vec![
            ProducedMedia {
                kind: MediaKind::Img("png".into()),
                generate: generate(),
                provider: None
            };
            2
        ],
        ..Default::default()
    };
    assert!(m.check(&two).is_err());
}
