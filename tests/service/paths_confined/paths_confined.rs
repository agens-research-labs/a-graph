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

/// A graph that names a file outside the folder of the session is refused with 422: `..` and absolute paths.
#[tokio::test]
async fn paths_confined() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    let at: Vec<&str> = body["errors"].as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap()).collect();
    for wanted in ["blackboards[0].source.file", "resources.media[0].source.path", "external.memory[0].path", "resources.prompts[2].source.path"] {
        assert!(at.contains(&wanted), "no error at `{wanted}` in {at:?}");
    }
    assert!(seen.lock().unwrap().is_empty());
}
