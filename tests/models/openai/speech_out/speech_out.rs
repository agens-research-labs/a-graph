use super::*;

/// A produced speech goes to the speech api.
#[tokio::test]
async fn speech_out() {
    let mut c = call("Have a wonderful day!");
    c.system = "cheerful".into();
    c.output_media = vec![ProducedMedia { kind: MediaKind::Audio("pcm16".into()), generate: Generate { voice: Some("alloy".into()), ..generate() }, provider: None }];
    let (r, req) = one_call(c, Resp::ok("RAWAUDIO")).await;
    assert_eq!(req.path, "/v1/audio/speech");
    assert_eq!(body(&req), json!({ "model": "gpt-5", "input": "Have a wonderful day!", "voice": "alloy", "response_format": "pcm", "instructions": "cheerful" }));
    let r = r.unwrap();
    assert_eq!((r.media[0].mime_type.as_str(), r.media[0].data.as_str()), ("audio/l16", STANDARD.encode("RAWAUDIO").as_str()));

    let mut m = call("x");
    m.output_media = vec![ProducedMedia { kind: MediaKind::Audio("mp3".into()), generate: Generate { voice: Some("alloy".into()), ..generate() }, provider: None }];
    let (r, _) = one_call(m, Resp::ok("ID3")).await;
    assert_eq!(r.unwrap().media[0].mime_type, "audio/mpeg");
}
