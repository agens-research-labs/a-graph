use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A web service gets a post with the arguments and the header.
#[tokio::test]
async fn http_post_header() {
    unsafe { std::env::set_var("A_GRAPH_TEST_TOOL_KEY", "secret-value") };
    let (_, service) = serve_at(18608, |_| Resp::ok(json!({ "temp": 22 }).to_string())).await;
    let (p, _) = program(here(), "http_post_header", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out)[0].1, r#"R({"temp":22})"#);
    let req = service.lock().unwrap()[0].clone();
    assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/weather"));
    assert_eq!(serde_json::from_str::<Value>(&req.body).unwrap(), json!({ "city": "Roma" }));
    assert!(req.headers.contains("x-key: secret-value"), "{}", req.headers);
}
