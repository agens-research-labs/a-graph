use super::*;

/// The workspace id is sent only when configured.
#[tokio::test]
async fn workspace_id() {
    let (_, req) = one_call(call("q"), Resp::ok(text_answer("x"))).await;
    assert!(!req.headers.contains("anthropic-workspace-id"), "{}", req.headers);

    let answer = Arc::new(Mutex::new(Some(Resp::ok(text_answer("x")))));
    let (base, seen) = serve(move |_| answer.lock().unwrap().take().unwrap()).await;
    let mut m = api_model(KEY_VAR);
    m.workspace_id = Some("wrkspc_demo".into());
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var(KEY_VAR, "test-key") };
    Anthropic::new(&m, &Path::root()).unwrap().with_base_url(&base).invoke(&call("q")).await.unwrap();
    assert!(seen.lock().unwrap()[0].headers.contains("anthropic-workspace-id: wrkspc_demo"));
}
