use super::*;

/// A produced speech goes to the speech api.
#[tokio::test]
async fn speech_out() {
    let mut c = call("Have a wonderful day!");
    c.output_media = vec![ProducedMedia { kind: MediaKind::Audio("pcm16".into()), generate: Generate { voice: Some("en_paul_neutral".into()), ..generate() }, provider: None }];
    let (r, req) = one_call(c, Resp::ok(json!({ "audio_data": "UkFX" }).to_string())).await;
    assert_eq!(req.path, "/v1/audio/speech");
    assert_eq!(body(&req), json!({ "model": "mistral-small-latest", "input": "Have a wonderful day!", "response_format": "pcm", "voice_id": "en_paul_neutral" }));
    let r = r.unwrap();
    assert_eq!((r.media[0].mime_type.as_str(), r.media[0].data.as_str()), ("audio/l16", "UkFX"));

    let mut m = call("x");
    m.output_media = vec![ProducedMedia { kind: MediaKind::Audio("mp3".into()), generate: generate(), provider: None }];
    let (r, req) = one_call(m, Resp::ok(json!({ "audio_data": "SUQz" }).to_string())).await;
    assert!(body(&req).get("voice_id").is_none());
    assert_eq!(r.unwrap().media[0].mime_type, "audio/mpeg");
}
