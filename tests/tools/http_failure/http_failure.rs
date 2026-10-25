use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A web service that fails tells the model the status and the body.
#[tokio::test]
async fn http_failure() {
    let (_, _) = serve_at(18609, |_| Resp::status(500, "overloaded")).await;
    let (p, _) = program(here(), "http_failure", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    let said = &items(&out)[0].1;
    assert!(said.starts_with("R(error: the service answered 500") && said.contains("overloaded"), "{said}");
}
