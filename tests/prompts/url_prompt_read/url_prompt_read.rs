use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A prompt from a url is read when the run starts, as a prompt file, and filled as any other.
#[tokio::test]
async fn url_prompt_read() {
    let text = "content:\n  system: \"Be brief.\"\n  user: \"Q: {{user_message}}\"\n";
    let (files, downloads) = listen(move |_| Resp::ok(text)).await;
    let (p, seen) = program_with(
        here(),
        "url_prompt_read",
        &[("files", files)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    Arc::new(p).run("why").await.unwrap();
    assert_eq!(downloads.lock().unwrap().len(), 1);
    let messages = sent(&seen)["messages"].clone();
    assert_eq!(messages[0]["content"], "Be brief.");
    assert_eq!(messages[1]["content"], "Q: why");
}
