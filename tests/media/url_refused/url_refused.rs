use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The failure of the download of a media stops the run before the first node, as `io` on its url; the message
/// names the media and never holds the url.
#[tokio::test]
async fn url_refused() {
    let cases = [
        (
            "url_refused",
            Resp::status(404, "no"),
            "media `doc`: the server answered 404",
        ),
        (
            "url_refused",
            Resp::bytes(vec![0xff, 0xfe]),
            "media `doc`: it is not text: ",
        ),
        (
            "url_refused",
            Resp::bytes(vec![b'a'; 64 * 1024 * 1024 + 1]),
            "media `doc`: the urls of the run passed 64 MiB together",
        ),
        (
            "ftp",
            Resp::ok("# the document"),
            "media `doc`: the scheme `ftp` is not http or https",
        ),
    ];
    for (name, answer, message) in cases {
        let answer = Arc::new(answer);
        let (files, _) = listen(move |_| Resp {
            status: answer.status,
            body: answer.body.clone(),
            location: None,
            drip: None,
            raw: answer.raw.clone(),
        })
        .await;
        let (p, seen) = program_with(here(), name, &[("files", files)], Resp::ok(chat("ok", None))).await;
        let e = Arc::new(p).run("summarise").await.unwrap_err();
        assert_eq!(
            (e.rule.code(), e.path.to_string().as_str()),
            ("io", "resources.media[0].source.url"),
            "{e}"
        );
        assert!(e.message.starts_with(message), "{}", e.message);
        assert!(!e.message.contains("secret"), "{}", e.message);
        assert!(seen.lock().unwrap().is_empty(), "{message}: the model is never called");
    }
}
