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

/// With `retention_seconds: 1` a run is forgotten about a second after it ends.
#[tokio::test]
async fn runs_forgotten() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" })).await;
    let id = body["run"].as_str().unwrap().to_string();
    s.finished(&id).await;
    assert_eq!(s.get(&format!("/runs/{id}")).await.0, 200, "still there just after the end");
    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
    assert_eq!(s.get(&format!("/runs/{id}")).await.0, 404);
}
