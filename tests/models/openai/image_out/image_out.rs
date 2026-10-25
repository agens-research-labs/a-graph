use super::*;

/// A produced image goes to the images api.
#[tokio::test]
async fn image_out() {
    let mut c = call("a red circle");
    c.system = "flat style".into();
    c.output_media = vec![image(
        "jpg",
        Generate { size: Some(Size { width: 1024, height: 1536 }), quality: Some("high".into()), background: Some(false), ..generate() },
        Some(MediaProvider::Openai(OpenaiMedia { moderation: Some(Moderation::Low), output_compression: Some(80), style: None })),
    )];
    let answer = json!({ "created": 1, "data": [{ "b64_json": "SU1H" }], "output_format": "jpeg", "usage": { "input_tokens": 10, "output_tokens": 1000 } }).to_string();
    let (r, req) = one_call(c.clone(), Resp::ok(answer.clone())).await;
    assert_eq!(req.path, "/v1/images/generations");
    assert_eq!(
        body(&req),
        json!({ "model": "gpt-5", "prompt": "flat style\n\na red circle", "n": 1, "output_format": "jpeg", "size": "1024x1536", "quality": "high", "background": "opaque", "moderation": "low", "output_compression": 80 })
    );
    let r = r.unwrap();
    assert_eq!((r.media[0].mime_type.as_str(), r.media[0].data.as_str()), ("image/jpeg", "SU1H"));
    assert_eq!((r.info.usage.input_tokens, r.info.usage.output_tokens), (10, 1000));
    assert!(r.message.is_none());

    // transparency
    let mut t = call("x");
    t.output_media = vec![image("png", Generate { background: Some(true), ..generate() }, None)];
    let (_, req) = one_call(t, Resp::ok(answer.clone())).await;
    assert_eq!(body(&req)["background"], "transparent");

    // the model made another format than the resource asks
    let (r, _) = one_call(c, Resp::ok(json!({ "data": [{ "b64_json": "SU1H" }], "output_format": "png" }).to_string())).await;
    assert!(matches!(&r, Err(ModelError::MimeMismatch { expected, produced, .. }) if expected == "image/jpeg" && produced == "image/png"), "{r:?}");
}
