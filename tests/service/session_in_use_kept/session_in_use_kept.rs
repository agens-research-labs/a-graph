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

/// A session with a run going on is not deleted, however old its folder is; it goes after the run, once idle.
#[tokio::test]
async fn session_in_use_kept() {
    let data = std::path::Path::new("target/test-service/session_in_use_kept");
    let _ = std::fs::remove_dir_all(data);
    let (model, _) = listen_slowly(std::time::Duration::from_secs(4), |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "s2" })).await;
    assert_eq!(status, 202, "{body}");
    let id = body["run"].as_str().unwrap().to_string();
    let started = std::time::SystemTime::now();
    // the retention (1 s) plus the scan interval (1 s) plus a margin: the sweep has looked at the folder
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    assert_eq!(s.get(&format!("/runs/{id}")).await.1["state"], "running", "the run is still going on");
    assert!(data.join("s2").is_dir(), "the folder stays while the run uses it");
    assert_eq!(s.finished(&id).await["state"], "done");
    // the end of the run stamps the mark again, about 4 seconds after the start
    let mark = data.join("s2/.a-graph-session");
    let restamped = wait_until(std::time::Duration::from_secs(5), || std::fs::metadata(&mark).and_then(|m| m.modified()).is_ok_and(|t| t > started + std::time::Duration::from_secs(3))).await;
    assert!(restamped, "the end of the run stamps the mark");
    assert!(wait_until(std::time::Duration::from_secs(10), || !data.join("s2").exists()).await, "the idle session is deleted after the run");
}
