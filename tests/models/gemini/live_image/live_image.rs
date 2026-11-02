use super::*;

/// Live image.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY; generates an image"]
async fn live_image() {
    let mut c = call("A small red circle on a white background.");
    // this model makes jpeg only: a png is refused by Google's own message
    c.output_media = vec![ProducedMedia {
        kind: MediaKind::Img("png".into()),
        generate: generate(),
        provider: None,
    }];
    let err = live("gemini-3.1-flash-image").invoke(&c).await.unwrap_err().to_string();
    assert!(err.contains("image/jpeg") && err.contains("(gemini)"), "{err}");

    c.output_media = vec![ProducedMedia {
        kind: MediaKind::Img("jpg".into()),
        generate: generate(),
        provider: None,
    }];
    let r = match live("gemini-3.1-flash-image").invoke(&c).await {
        Ok(r) => r,
        // image generation has no quota on the free tier: the call cannot be verified with such a key
        Err(e) if e.to_string().contains("Free Tier") => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    assert_eq!(r.media.len(), 1);
    assert_eq!(r.media[0].mime_type, "image/jpeg");
    assert!(r.media[0].data.starts_with("/9j/"), "not a JPEG");
}
