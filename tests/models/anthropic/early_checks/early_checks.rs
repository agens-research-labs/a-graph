use super::*;

/// Max tokens and image format are checked before any call.
#[tokio::test]
async fn early_checks() {
    let m = client("http://127.0.0.1:1");
    let mut c = call("q");
    c.max_tokens = None;
    assert!(matches!(m.invoke(&c).await, Err(ModelError::Unsupported { .. })));
    let mut c = call("q");
    c.media = vec![MediaInput::Image { format: "bmp".into(), data: "x".into() }];
    assert!(matches!(m.invoke(&c).await, Err(ModelError::Unsupported { .. })));
}
