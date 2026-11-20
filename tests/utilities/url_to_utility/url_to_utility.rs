use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media from a url reaches its utility as the bytes the url answered, with its `mime_type` or else the type of the
/// extension of the path of the url (the query left out), and the model gets what the utility made of it.
#[tokio::test]
async fn url_to_utility() {
    for name in ["url_to_utility", "with_mime"] {
        through_utility(name).await;
    }
}

async fn through_utility(name: &str) {
    let (files, _) = listen(|_| Resp::bytes(b"%PDF-1.7 bytes".to_vec())).await;
    let (utility, got) = listen(|_| Resp::ok(r#"{"text":"converted"}"#)).await;
    let (p, seen) = program_with(
        here(),
        name,
        &[("files", files), ("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    Arc::new(p).run("x").await.unwrap();
    let asked: Value = serde_json::from_str(&got.lock().unwrap()[0].body).unwrap();
    assert_eq!(
        asked,
        json!({ "data": STANDARD.encode(b"%PDF-1.7 bytes"), "mime_type": "application/pdf" })
    );
    let sent: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert!(
        sent["messages"][1]["content"]
            .as_str()
            .unwrap()
            .starts_with("converted"),
        "{sent}"
    );
}
