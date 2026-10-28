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

/// With `run_timeout_seconds: 1` a run whose model takes 3 seconds ends as failed with `run_timeout`.
#[tokio::test]
async fn run_timeout() {
    serve_at_slowly(17404, std::time::Duration::from_secs(3), |_| Resp::ok(answer("too late"))).await;
    let s = start(&config_path(), 17405).await;
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    assert_eq!(status, 202, "{body}");
    let id = body["run"].as_str().unwrap().to_string();
    let state = s.finished(&id).await;
    assert_eq!(state["state"], "failed", "{state}");
    assert_eq!(state["errors"][0]["rule"], "run_timeout", "{state}");
    let events = s.events(&id).await;
    let (name, data) = events.last().unwrap();
    assert_eq!((name.as_str(), &data["errors"][0]["rule"]), ("error", &json!("run_timeout")));
}
