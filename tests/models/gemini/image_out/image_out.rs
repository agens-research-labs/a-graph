use super::*;

/// A produced image.
#[tokio::test]
async fn image_out() {
    let mut c = call("a cover");
    c.output_media = vec![ProducedMedia {
        kind: MediaKind::Img("jpg".into()),
        generate: generate(),
        provider: Some(MediaProvider::Gemini(GeminiMedia {
            image_size: Some(GeminiImageSize::K2),
            aspect_ratio: Some("16:9".into()),
        })),
    }];
    let answer = interaction(
        "completed",
        json!([{ "type": "model_output", "content": [{ "type": "image", "mime_type": "image/jpeg", "data": "SU1H" }] }]),
    );
    let (r, req) = one_call(c.clone(), Resp::ok(answer)).await;
    assert_eq!(
        body(&req)["response_format"],
        json!({ "type": "image", "mime_type": "image/jpeg", "image_size": "2K", "aspect_ratio": "16:9" })
    );
    let r = r.unwrap();
    assert_eq!(r.media.len(), 1);
    assert_eq!(
        (r.media[0].mime_type.as_str(), r.media[0].data.as_str()),
        ("image/jpeg", "SU1H")
    );
    assert!(r.message.is_none());

    // the model made another type than the resource asks
    let wrong = interaction(
        "completed",
        json!([{ "type": "model_output", "content": [{ "type": "image", "mime_type": "image/png", "data": "SU1H" }] }]),
    );
    let (r, _) = one_call(c, Resp::ok(wrong)).await;
    assert!(
        matches!(&r, Err(ModelError::MimeMismatch { expected, produced, .. }) if expected == "image/jpeg" && produced == "image/png"),
        "{r:?}"
    );
}
