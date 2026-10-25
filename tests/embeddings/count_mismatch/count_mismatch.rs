use crate::common::embeddings::*;
use crate::common::http::{Resp, serve_at};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server that answers with fewer vectors than texts is an invalid response.
#[tokio::test]
async fn count_mismatch() {
    serve_at(18903, |_| Resp::ok(r#"{"embeddings":[[1.0,2.0]]}"#)).await;
    let m = manager(&own_yml(here()));
    let e = m.embed("emb_gemma", &["a".to_string(), "b".to_string()]).await.unwrap_err();
    assert!(matches!(&e, ModelError::InvalidResponse { .. }), "{e}");
}
