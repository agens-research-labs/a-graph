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

/// `POST /run` with the yml of a graph and a message answers the same JSON as the command line.
#[tokio::test]
async fn run_ok() {
    let (model, seen) = listen(|_| Resp::ok(answer("the lease lasts 12 months"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body, json!({ "items": [{ "node": "b", "message": "the lease lasts 12 months", "media": [], "reasoning": [] }] }));
    let first: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(first["messages"][1]["content"], "Q: how long?");
}
