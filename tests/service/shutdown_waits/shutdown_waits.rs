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

/// On `SIGTERM` the service takes no new request, lets the run going on end, then exits with 0. The model
/// answers in a second: its second call (node b) can only come after the signal, from a run that went on.
#[cfg(unix)]
#[tokio::test]
async fn shutdown_waits() {
    let (_, seen) = serve_at_slowly(17409, std::time::Duration::from_secs(1), |_| Resp::ok(answer("ok"))).await;
    let mut s = start(&config_path(), 17410).await;
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    assert_eq!(status, 202, "{body}");
    // the run is going on once the model is called: then the signal
    for _ in 0..100 {
        if !seen.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(seen.lock().unwrap().len(), 1, "the model was not called");
    s.terminate();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(reqwest::get("http://127.0.0.1:17410/health").await.is_err(), "the service still takes requests");
    assert_eq!(seen.lock().unwrap().len(), 1, "the first call is still going on");
    assert_eq!(s.exited(10).await, Some(0));
    assert_eq!(seen.lock().unwrap().len(), 2, "the run went on to node b after the signal");
}
