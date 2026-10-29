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

/// The model answers 500: the state is `failed` and the stream ends with an `error` event.
#[tokio::test]
async fn runs_fails() {
    let (model, _) = listen(|_| Resp::status(500, "broken")).await;
    let s = start(&config_path()).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" })).await;
    let id = body["run"].as_str().unwrap().to_string();
    let events = s.events(&id).await;
    let last = events.last().unwrap();
    assert_eq!(last.0, "error");
    assert_eq!(last.1["errors"][0]["rule"], "model_failed");
    let state = s.finished(&id).await;
    assert_eq!(state["state"], "failed");
    assert_eq!(state["errors"][0]["rule"], "model_failed");
}
