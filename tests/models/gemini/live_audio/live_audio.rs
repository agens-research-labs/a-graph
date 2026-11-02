use super::*;

/// Live audio.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY; generates audio"]
async fn live_audio() {
    let mut c = call("Hello!");
    c.output_media = vec![ProducedMedia {
        kind: MediaKind::Audio("wav".into()),
        generate: Generate {
            voice: Some("Kore".into()),
            ..generate()
        },
        provider: None,
    }];
    let r = live("gemini-3.8-flash-tts").invoke(&c).await.unwrap();
    assert_eq!(r.media[0].mime_type, "audio/wav");
    assert!(r.media[0].data.starts_with("UklGR"), "not a RIFF/WAV file");
}
