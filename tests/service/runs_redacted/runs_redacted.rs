use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// The passport forbids contents (internal data): the stream keeps the facts and drops the texts.
#[tokio::test]
async fn runs_redacted() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (_, body) = s
        .start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" }))
        .await;
    let events = s.events(body["run"].as_str().unwrap()).await;
    let call = events.iter().find(|(n, _)| n == "model.call").unwrap();
    assert_eq!(call.1["content_redacted"], true);
    assert!(
        call.1.get("prompt").is_none() && call.1.get("answer").is_none(),
        "{}",
        call.1
    );
    assert_eq!(call.1["input_tokens"], 1);
}
