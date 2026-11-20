use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Only the urls a node uses are downloaded: not a media, a prompt or the summary prompt of a memory that no node
/// reaches.
#[tokio::test]
async fn url_unused_skipped() {
    let (files, downloads) = listen(|_| Resp::ok("# the document")).await;
    let (p, _) = program_with(
        here(),
        "url_unused_skipped",
        &[("files", files)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    Arc::new(p).run("summarise").await.unwrap();
    let paths: Vec<String> = downloads.lock().unwrap().iter().map(|r| r.path.clone()).collect();
    assert_eq!(paths, ["/doc.md"]);
}
