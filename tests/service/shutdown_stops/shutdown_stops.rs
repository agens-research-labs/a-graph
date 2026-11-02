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

/// On `SIGTERM` a run that would last longer than the 30 seconds of the shutdown is stopped then, and still ends
/// as any run: its log gets the last event, with `service_stopped`. The callers still connected are answered
/// before the service exits: the one waiting on `POST /run` gets the 502, the watcher of the events gets the
/// `error`. The service exits soon after, with 0.
#[cfg(unix)]
#[tokio::test]
async fn shutdown_stops() {
    let log = "target/test-service/shutdown_stops/default/run.log";
    let _ = std::fs::remove_file(log);
    let (model, seen) = listen_slowly(std::time::Duration::from_secs(120), |_| Resp::ok(answer("too late"))).await;
    let mut s = start(&config_path()).await;
    let (status, body) = s
        .start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" }))
        .await;
    assert_eq!(status, 202, "{body}");
    let port = s.port;
    let id = body["run"].as_str().unwrap().to_string();
    // a caller waiting on POST /run and a watcher of the events of the other run, both still connected at the deadline
    let waiting = tokio::spawn(async move {
        let r = reqwest::Client::new()
            .post(format!("http://127.0.0.1:{port}/run"))
            .json(&json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" }))
            .send()
            .await
            .unwrap();
        (r.status().as_u16(), r.json::<Value>().await.unwrap_or(Value::Null))
    });
    let watching = tokio::spawn(async move {
        let text = reqwest::get(format!("http://127.0.0.1:{port}/runs/{id}/events"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        parse_sse(&text)
    });
    // the runs are going on once the model is called: then the signal
    for _ in 0..100 {
        if seen.lock().unwrap().len() == 2 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(seen.lock().unwrap().len(), 2, "the model was not called twice");
    let signalled = std::time::Instant::now();
    s.terminate();
    assert_eq!(s.exited(40).await, Some(0));
    assert!(
        signalled.elapsed() >= std::time::Duration::from_secs(30),
        "the run was stopped before the deadline: {:?}",
        signalled.elapsed()
    );
    let lines = log_lines(log);
    let last = lines.last().unwrap_or_else(|| panic!("the log is empty"));
    assert_eq!(last["code"], "service_stopped", "{lines:?}");
    let (status, body) = waiting.await.unwrap();
    assert_eq!(status, 502, "{body}");
    assert_eq!(body["errors"][0]["rule"], "service_stopped", "{body}");
    let events = watching.await.unwrap();
    let (name, data) = events.last().unwrap_or_else(|| panic!("no event was watched"));
    assert_eq!(name, "error", "{events:?}");
    assert_eq!(data["errors"][0]["rule"], "service_stopped", "{data}");
}
