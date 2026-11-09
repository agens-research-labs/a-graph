use crate::common::fresh_copy;
use crate::common::run_node::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the server of the embeddings wrote when a memory looks for similar exchanges, or saves one,
/// stays out of the message: it is the detail.
#[tokio::test]
async fn embedding_error() {
    // `ok` calls answer a vector, then every call fails: 0 fails the reading, 1 the saving
    for ok in [0, 1] {
        let path = "target/test-memory/embedding_error.json";
        fresh_copy(here(), "files/session.json", path);
        let calls = AtomicUsize::new(0);
        let (embedding_port, _) = listen(move |_| {
            if calls.fetch_add(1, Ordering::SeqCst) < ok {
                Resp::ok(r#"{"embeddings":[[0.0,1.0]]}"#)
            } else {
                Resp::status(500, r#"{"error":"the patient has diabetes"}"#)
            }
        })
        .await;
        let (p, _) = program_with(
            here(),
            "embedding_error",
            &[("embedding", embedding_port)],
            Resp::ok(chat("ok", None)),
        )
        .await;
        let e = p.run_node(entity(&p), user("when do I pay?")).await.unwrap_err();
        assert_eq!(e.rule.code(), "model_failed", "{ok}: {e:?}");
        assert!(!e.message.contains("diabetes"), "{ok}: {}", e.message);
        assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"), "{ok}");
    }
}
