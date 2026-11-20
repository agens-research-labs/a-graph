use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A markdown media from a url is downloaded once when the run starts, with its header, and goes with the question
/// of every node that reads it.
#[tokio::test]
async fn url_md_read() {
    // SAFETY: no other test reads or writes this variable.
    unsafe { std::env::set_var("A_GRAPH_URL_MD_READ", "Bearer t") };
    let (files, downloads) = listen(|_| Resp::ok("# the document")).await;
    let (p, seen) = program_with(here(), "url_md_read", &[("files", files)], Resp::ok(chat("ok", None))).await;
    Arc::new(p).run("summarise").await.unwrap();
    let downloads = downloads.lock().unwrap();
    assert_eq!(downloads.len(), 1, "one download for the two nodes");
    assert_eq!(downloads[0].path, "/doc.md?sig=secret");
    assert!(
        downloads[0].headers.contains("authorization: bearer t"),
        "{}",
        downloads[0].headers
    );
    let seen = seen.lock().unwrap();
    let asked: Vec<Value> = seen.iter().map(|r| serde_json::from_str(&r.body).unwrap()).collect();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0]["messages"][0]["content"], "# the document\n\nsummarise");
    assert_eq!(asked[1]["messages"][0]["content"], "# the document\n\nok");
}
