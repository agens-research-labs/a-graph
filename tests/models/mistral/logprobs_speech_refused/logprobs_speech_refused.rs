use super::*;

/// A produced speech goes to an API without logprobs: asking them for it is refused.
#[tokio::test]
async fn logprobs_speech_refused() {
    let m = client("http://unused");
    let req = ModelRequirements {
        logprobs: Some(0),
        output_media: vec![ProducedMedia {
            kind: MediaKind::Audio("mp3".into()),
            generate: generate(),
            provider: None,
        }],
        ..Default::default()
    };
    let ModelError::Unsupported { problems, .. } = m.check(&req).unwrap_err() else {
        panic!("not Unsupported")
    };
    assert!(problems.iter().any(|p| p.contains("gives no logprobs")), "{problems:?}");
}
