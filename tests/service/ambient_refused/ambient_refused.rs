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

/// A model with `credentials: ambient` is refused when its provider is not in `allowed_ambient`;
/// one whose provider is listed, and one with a key of its own, are not.
#[tokio::test]
async fn ambient_refused() {
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[]), "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    let errors: Vec<(&str, &str)> = body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["rule"].as_str().unwrap(), e["path"].as_str().unwrap()))
        .collect();
    assert_eq!(errors, [("ambient_not_allowed", "models[0].credentials")]);
}
