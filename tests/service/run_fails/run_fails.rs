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

/// The model answers 500: the run fails and the service answers 502 with the error.
#[tokio::test]
async fn run_fails() {
    let (model, _) = listen(|_| Resp::status(500, "broken")).await;
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "x" })).await;
    assert_eq!(status, 502);
    assert_eq!(body["errors"][0]["rule"], "model_failed");
    assert!(body["errors"][0]["message"].as_str().unwrap().contains("500"), "{body}");
}
