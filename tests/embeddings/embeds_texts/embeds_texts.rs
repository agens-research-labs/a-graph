use crate::common::embeddings::*;
use crate::common::http::{Resp, listen};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The texts go to `POST /api/embed` with the model of the yml; one vector comes back per text.
#[tokio::test]
async fn embeds_texts() {
    let (embedding_port, seen) = listen(|_| Resp::ok(r#"{"model":"embeddinggemma","embeddings":[[0.5,0.25,1.0],[-1.0,0.0,2.0]]}"#)).await;
    let m = manager(&yml_with(here(), "embeds_texts", &[("embedding", embedding_port)]));
    let texts = vec!["ciao mondo".to_string(), "hello".to_string()];
    let vectors = m.embed("emb_gemma", &texts).await.unwrap();
    assert_eq!(vectors, vec![vec![0.5, 0.25, 1.0], vec![-1.0, 0.0, 2.0]]);

    let seen = seen.lock().unwrap();
    assert_eq!((seen.len(), seen[0].method.as_str(), seen[0].path.as_str()), (1, "POST", "/api/embed"));
    let body: serde_json::Value = serde_json::from_str(&seen[0].body).unwrap();
    assert_eq!(body["model"], "embeddinggemma");
    assert_eq!(body["input"], serde_json::json!(["ciao mondo", "hello"]));
}
