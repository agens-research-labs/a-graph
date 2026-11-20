use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A text from the url of a prompt that is not a prompt file, that has an alias or a tag, or whose placeholders the
/// node does not give, stops the run before the first node; the error never holds the url.
#[tokio::test]
async fn url_prompt_refused() {
    let cases = [
        ("content: [a]\n", "bad_value", "resources.prompts[0].source.url"),
        (
            "x: &a \"Be brief.\"\ncontent:\n  system: *a\n  user: \"{{user_message}}\"\n",
            "not_supported",
            "resources.prompts[0].source.url",
        ),
        (
            "content:\n  system: !t \"Be brief.\"\n  user: \"{{user_message}}\"\n",
            "not_supported",
            "resources.prompts[0].source.url",
        ),
        (
            "content:\n  system: \"Be brief.\"\n  user: \"{{user_message}} {{missing}}\"\n",
            "placeholder_unbound",
            "graph.nodes[0].resources.prompt",
        ),
    ];
    for (text, rule, path) in cases {
        let (files, _) = listen(move |_| Resp::ok(text)).await;
        let (p, seen) = program_with(
            here(),
            "url_prompt_refused",
            &[("files", files)],
            Resp::ok(chat("ok", None)),
        )
        .await;
        let e = Arc::new(p).run("why").await.unwrap_err();
        assert_eq!(
            (e.rule.code(), e.path.to_string().as_str()),
            (rule, path),
            "{text}: {e}"
        );
        assert!(!e.message.contains("secret"), "{}", e.message);
        assert!(seen.lock().unwrap().is_empty(), "{text}: the model is never called");
    }
}
