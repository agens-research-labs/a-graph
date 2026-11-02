use crate::common::embeddings::*;
use crate::common::http::{Resp, listen};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server that answers with fewer vectors than texts is an invalid response.
#[tokio::test]
async fn count_mismatch() {
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[1.0,2.0]]}"#)).await;
    let m = manager(&yml_with(here(), "count_mismatch", &[("embedding", embedding_port)]));
    let e = m
        .embed("emb_gemma", &["a".to_string(), "b".to_string()])
        .await
        .unwrap_err();
    assert!(matches!(&e, ModelError::InvalidResponse { .. }), "{e}");
}
