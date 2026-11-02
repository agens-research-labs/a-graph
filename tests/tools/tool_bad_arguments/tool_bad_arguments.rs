use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Arguments that do not match the parameters of a tool are told to the model; the tool is not called.
#[tokio::test]
async fn tool_bad_arguments() {
    let (p, _) = program(
        here(),
        "tool_bad_arguments",
        model(vec![("t", json!({ "town": "Roma" }))], false),
    )
    .await;
    let out = p.run("hi").await.unwrap();
    let answer = &items(&out)[0].1;
    assert!(
        answer.starts_with("R(error: the arguments do not match the parameters of `t`"),
        "{answer}"
    );
    assert!(!answer.contains("got"), "{answer}");
}
