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

/// A log url whose host the graph does not allow is refused before the log is built: that host
/// gets nothing, not even the errors of the build with the header the service lets it use.
#[tokio::test]
async fn log_host_refused() {
    // SAFETY: no other test reads or writes this variable.
    unsafe { std::env::set_var("A_GRAPH_LOG_HOST_REFUSED", "the-secret") };
    let (sink, seen) = listen(|_| Resp::ok("")).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("sink", sink), ("model", 1)]), "message": "x" }))
        .await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(
        errors(&body),
        [(
            "network_not_allowed".to_string(),
            "observability.output.url".to_string()
        )]
    );
    // the log is delivered by a task of its own: it is given the time to send what it had
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let seen = seen.lock().unwrap();
    assert!(
        seen.is_empty(),
        "the refused host got {} request(s), the first with the headers:\n{}",
        seen.len(),
        seen[0].headers
    );
}
