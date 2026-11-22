use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A url answered with gzip is asked for compressed and read as the text it holds.
#[tokio::test]
async fn url_gzip_read() {
    // SAFETY: no other test reads or writes this variable.
    unsafe { std::env::set_var("A_GRAPH_URL_GZIP_READ", "Bearer t") };
    let (files, downloads) = listen(|_| Resp::gzip(b"# the document")).await;
    let (p, seen) = program_with(here(), "url_gzip_read", &[("files", files)], Resp::ok(chat("ok", None))).await;
    Arc::new(p).run("summarise").await.unwrap();
    let downloads = downloads.lock().unwrap();
    assert!(
        downloads[0].headers.contains("accept-encoding: gzip,deflate"),
        "{}",
        downloads[0].headers
    );
    let seen = seen.lock().unwrap();
    let asked: Value = serde_json::from_str(&seen[0].body).unwrap();
    assert_eq!(asked["messages"][0]["content"], "# the document\n\nsummarise");
}
