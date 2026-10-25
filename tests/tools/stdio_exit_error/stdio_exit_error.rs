use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program that exits with an error tells the model why.
#[tokio::test]
async fn stdio_exit_error() {
    let (p, _) = program(here(), "stdio_exit_error", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: boom)");
}
