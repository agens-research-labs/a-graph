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

/// A graph whose node cites a model that is not declared is refused with 422 and its errors.
#[tokio::test]
async fn invalid_yml() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 422);
    let first = &body["errors"][0];
    assert_eq!(
        (first["rule"].as_str(), first["path"].as_str()),
        (Some("unknown_ref"), Some("graph.nodes[0].model.id"))
    );
    assert!(seen.lock().unwrap().is_empty(), "nothing was run");
}
