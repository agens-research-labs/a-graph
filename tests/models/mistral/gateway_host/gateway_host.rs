use super::*;

/// With a `host`, the calls go to that server instead of the API of the provider, with the paths of the API under
/// it; a final `/` is not doubled.
#[tokio::test]
async fn gateway_host() {
    let (base, seen) = serve(|_| Resp::ok(text_answer("hello"))).await;
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var(KEY_VAR, "test-key") };
    let mut m = api_model(KEY_VAR);
    m.host = Some(format!("{base}/gateway"));
    let r = Mistral::new(&m, &Path::root().key("models").idx(0))
        .unwrap()
        .invoke(&call("q"))
        .await;

    assert_eq!(r.unwrap().message.as_deref(), Some("hello"));
    let req = seen.lock().unwrap()[0].clone();
    assert_eq!(
        (req.method.as_str(), req.path.as_str()),
        ("POST", "/gateway/v1/chat/completions")
    );
}
