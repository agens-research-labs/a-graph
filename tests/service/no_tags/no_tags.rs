use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// The graph `<name>.yml` is refused with 422 for its YAML tag, and nothing is run.
async fn refused(name: &str) {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read(&format!("{name}.yml"), &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 422, "{body}");
    let errors = body["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{body}");
    assert_eq!(errors[0]["rule"], "not_supported", "{body}");
    assert!(errors[0]["message"].as_str().unwrap().contains("tags"), "{body}");
    assert!(seen.lock().unwrap().is_empty(), "nothing was run");
}

/// A prompt read from a tagged path is refused.
#[tokio::test]
async fn no_tags_prompt() {
    refused("prompt").await;
}

/// A log written to a tagged path is refused.
#[tokio::test]
async fn no_tags_log() {
    refused("log").await;
}

/// A memory kept at a tagged path is refused.
#[tokio::test]
async fn no_tags_memory() {
    refused("memory").await;
}

/// Tagged `allowed_hosts` are refused.
#[tokio::test]
async fn no_tags_hosts() {
    refused("hosts").await;
}

/// `!` inside a text is not a tag: the graph runs.
#[tokio::test]
async fn no_tags_in_texts() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("plain.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 200, "{body}");
    let first: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(first["messages"][1]["content"], "!x !!str ! x");
}
