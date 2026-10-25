use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program that does not answer with json tells the model.
#[tokio::test]
async fn stdio_not_json() {
    let (p, _) = program(here(), "stdio_not_json", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    let said = &items(&out)[0].1;
    assert!(said.starts_with("R(error: the tool did not answer with JSON") && said.contains("hello there"), "{said}");
}
