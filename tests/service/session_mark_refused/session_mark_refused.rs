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

/// A graph that names a file `.a-graph-session` at the top of the folder of the session, the mark the service empties
/// at every use, is refused with 422: after `./` and in any case too. The same name in a subfolder is another file.
#[tokio::test]
async fn session_mark_refused() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 422, "{body}");
    let mut at: Vec<&str> = body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["path"].as_str().unwrap())
        .collect();
    at.sort();
    assert_eq!(
        at,
        [
            "blackboards[0].source.file",
            "external.memory[0].path",
            "observability.output.file"
        ],
        "{body}"
    );
    assert!(
        body["errors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["rule"] == "not_supported"),
        "{body}"
    );
    assert!(seen.lock().unwrap().is_empty());
}
