use super::*;

/// A produced audio.
#[tokio::test]
async fn audio_out() {
    let mut c = call("Have a wonderful day!");
    c.output_media = vec![ProducedMedia {
        kind: MediaKind::Audio("wav".into()),
        generate: Generate {
            voice: Some("Kore".into()),
            ..generate()
        },
        provider: None,
    }];
    // the audio item has no mime_type: it is the one asked for
    let answer = interaction(
        "completed",
        json!([{ "type": "model_output", "content": [{ "type": "audio", "data": "UklGRg==" }] }]),
    );
    let (r, req) = one_call(c, Resp::ok(answer)).await;
    let b = body(&req);
    assert_eq!(
        b["response_format"],
        json!({ "type": "audio", "mime_type": "audio/wav" })
    );
    assert_eq!(b["generation_config"]["speech_config"], json!([{ "voice": "Kore" }]));
    let r = r.unwrap();
    assert_eq!(
        (r.media[0].mime_type.as_str(), r.media[0].data.as_str()),
        ("audio/wav", "UklGRg==")
    );

    let mut c = call("x");
    c.output_media = vec![ProducedMedia {
        kind: MediaKind::Audio("pcm16".into()),
        generate: generate(),
        provider: None,
    }];
    let (_, req) = one_call(c, Resp::ok(text_answer("x"))).await;
    assert_eq!(body(&req)["response_format"]["mime_type"], "audio/l16");
}
