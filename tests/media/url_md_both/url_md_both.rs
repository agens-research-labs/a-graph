use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A markdown media from a url that one node reads as it is and another through a utility is downloaded once: the
/// first model gets its text, the utility the same bytes the url answered.
#[tokio::test]
async fn url_md_both() {
    let (files, downloads) = listen(|_| Resp::ok("# the document")).await;
    let (utility, got) = listen(|_| Resp::ok(r#"{"text":"converted"}"#)).await;
    let (p, seen) = program_with(
        here(),
        "url_md_both",
        &[("files", files), ("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    Arc::new(p).run("summarise").await.unwrap();
    assert_eq!(downloads.lock().unwrap().len(), 1, "one download for the two nodes");
    let asked: Value = serde_json::from_str(&got.lock().unwrap()[0].body).unwrap();
    assert_eq!(asked["data"], STANDARD.encode(b"# the document"));
    let seen = seen.lock().unwrap();
    let first: Value = serde_json::from_str(&seen[0].body).unwrap();
    assert_eq!(first["messages"][0]["content"], "# the document\n\nsummarise");
    let second: Value = serde_json::from_str(&seen[1].body).unwrap();
    assert_eq!(second["messages"][0]["content"], "converted\n\nok");
}
