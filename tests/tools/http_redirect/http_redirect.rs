use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A redirect is not followed: its host was never checked against `allowed_hosts`. The model is
/// told the status, and the target of the redirect receives nothing.
#[tokio::test]
async fn http_redirect() {
    let (_, elsewhere) = serve_at(17453, |_| Resp::ok("{}")).await;
    let (_, _) = serve_at(17452, |_| Resp::redirect(302, "http://127.0.0.1:17453/weather")).await;
    let (p, _) = program(here(), "http_redirect", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    let said = &items(&out)[0].1;
    assert!(said.starts_with("R(error: the service answered 302"), "{said}");
    assert!(elsewhere.lock().unwrap().is_empty());
}
