use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// With `retention_seconds: 1` a run is forgotten about a second after it ends.
#[tokio::test]
async fn runs_forgotten() {
    serve_at(19948, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19950).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    let id = body["run"].as_str().unwrap().to_string();
    s.finished(&id).await;
    assert_eq!(s.get(&format!("/runs/{id}")).await.0, 200, "still there just after the end");
    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
    assert_eq!(s.get(&format!("/runs/{id}")).await.0, 404);
}
