use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The download of a media and of a prompt is an event each, with the id and the bytes; the url is never written.
#[tokio::test]
async fn url_read_logged() {
    crate::common::clean_output("target/test-output/url_read_logged.log");
    let (files, _) = listen(|r| {
        if r.path.starts_with("/doc.md") {
            Resp::ok("# the document")
        } else {
            Resp::ok("content:\n  system: \"Be brief.\"\n  user: \"Q: {{user_message}}\"\n")
        }
    })
    .await;
    let (p, _) = program_with(
        here(),
        "url_read_logged",
        &[("files", files)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    Arc::new(p).run("why").await.unwrap();
    let lines = crate::common::run::log_lines("url_read_logged");
    let read = |code: &str| {
        let l = lines
            .iter()
            .find(|l| l["code"] == code)
            .unwrap_or_else(|| panic!("a {code} event"));
        (
            l["id"].as_str().map(str::to_string),
            l["bytes"].as_u64(),
            l["duration"].is_number(),
        )
    };
    assert_eq!(read("read.media"), (Some("doc".into()), Some(14), true));
    assert_eq!(read("read.prompt"), (Some("pr".into()), Some(61), true));
    assert!(
        lines.iter().all(|l| !l.to_string().contains("secret")),
        "the url is never in the log"
    );
}
