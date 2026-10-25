use crate::common::embeddings::*;
use crate::common::http::{Resp, serve_at};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An error of the server comes back as a provider error with its message and status.
#[tokio::test]
async fn server_error() {
    serve_at(18902, |_| Resp::status(404, r#"{"error":"model \"embeddinggemma\" not found"}"#)).await;
    let m = manager(&own_yml(here()));
    let e = m.embed("emb_gemma", &["a".to_string()]).await.unwrap_err();
    assert!(matches!(&e, ModelError::Provider { status: Some(404), message, .. } if message.contains("not found")), "{e}");
}
