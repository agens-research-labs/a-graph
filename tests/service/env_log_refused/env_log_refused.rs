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

/// The (rule, path) of every error of the answer.
fn errors(body: &Value) -> Vec<(String, String)> {
    body["errors"]
        .as_array()
        .unwrap_or_else(|| panic!("no errors in {body}"))
        .iter()
        .map(|e| {
            (
                e["rule"].as_str().unwrap().to_string(),
                e["path"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// A log header whose variable the service does not list is refused alone, before the log is
/// built: the log host, which the graph may reach, gets nothing.
#[tokio::test]
async fn env_log_refused() {
    // SAFETY: no other test reads or writes this variable.
    unsafe { std::env::set_var("A_GRAPH_ENV_LOG_REFUSED", "the-secret") };
    let (sink, seen) = listen(|_| Resp::ok("")).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("sink", sink), ("model", 1)]), "message": "x" }))
        .await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(
        errors(&body),
        [(
            "env_not_allowed".to_string(),
            "observability.output.headers.Authorization".to_string()
        )]
    );
    // the log is delivered by a task of its own: it is given the time to send what it had
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    assert!(seen.lock().unwrap().is_empty(), "the log host got a request");
}
