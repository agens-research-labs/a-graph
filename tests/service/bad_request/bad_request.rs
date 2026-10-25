use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// Requests that are not a JSON object with `yml`, `message` and `session` are refused with 400.
#[tokio::test]
async fn bad_request() {
    let s = start(&config_path(), 19972).await;
    let (status, body) = s.run_raw("not json").await;
    assert_eq!(status, 400);
    assert_eq!(body["errors"][0]["rule"], "bad_value");
    let (status, _) = s.run(json!({ "yml": read("graph.yml") })).await;
    assert_eq!(status, 400, "no message");
    let (status, body) = s.run(json!({ "yml": read("graph.yml"), "message": "x", "env": { "A": "b" } })).await;
    assert_eq!(status, 400, "there is no `env` in a request: the keys are variables of the container");
    assert!(body["errors"][0]["message"].as_str().unwrap().contains("env"), "{body}");
    let (status, _) = s.run(json!({ "yml": read("graph.yml"), "message": "x", "session": "../other" })).await;
    assert_eq!(status, 400, "a session is a name, not a path");
}
