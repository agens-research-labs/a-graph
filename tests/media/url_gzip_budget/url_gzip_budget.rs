use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The budget of 64 MiB counts the bytes of a url after decompression: a body far smaller on the network that holds
/// one byte more stops the run before the first node, as `io` on its url.
#[tokio::test]
async fn url_gzip_budget() {
    let sent = Resp::gzip(&vec![b'a'; 64 * 1024 * 1024 + 1]).raw.unwrap();
    assert!(sent.len() < 1024 * 1024, "the network carries less than 1 MiB");
    let (files, _) = listen(move |_| Resp {
        encoding: Some("gzip"),
        ..Resp::bytes(sent.clone())
    })
    .await;
    let (p, seen) = program_with(
        here(),
        "url_gzip_budget",
        &[("files", files)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let e = Arc::new(p).run("summarise").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string().as_str(), e.message.as_str()),
        (
            "io",
            "resources.media[0].source.url",
            "media `doc`: the urls of the run passed 64 MiB together"
        )
    );
    assert!(seen.lock().unwrap().is_empty(), "the model is never called");
}
