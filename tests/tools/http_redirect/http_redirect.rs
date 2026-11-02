use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A redirect is not followed: its host was never checked against `allowed_hosts`. The model is
/// told the status, and the target of the redirect receives nothing.
#[tokio::test]
async fn http_redirect() {
    let (elsewhere_port, elsewhere) = listen(|_| Resp::ok("{}")).await;
    let (tool_port, _) = listen(move |_| Resp::redirect(302, format!("http://127.0.0.1:{elsewhere_port}/weather"))).await;
    let (p, _) = program_with(here(), "http_redirect", &[("tool", tool_port)], model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    let said = &items(&out)[0].1;
    assert!(said.starts_with("R(error: the service answered 302"), "{said}");
    assert!(elsewhere.lock().unwrap().is_empty());
}
