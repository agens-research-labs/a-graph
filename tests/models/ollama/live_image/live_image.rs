use super::*;

/// Live image.
#[tokio::test]
#[ignore = "needs a local Ollama with gemma4:e2b"]
async fn live_image() {
    let mut c = call("What color is this image? One word.");
    c.max_tokens = Some(2000);
    c.media = vec![MediaInput::Image { format: "png".into(), data: RED_PNG.into() }];
    assert!(live("gemma4:e2b").invoke(&c).await.unwrap().message.unwrap().to_lowercase().contains("red"));
}
