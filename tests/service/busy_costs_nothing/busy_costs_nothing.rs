use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

/// The headers of a request whose body never comes, then the whole answer, read until the service closes the
/// connection, within `seconds`.
async fn without_body(port: u16, route: &str, seconds: u64) -> String {
    let mut conn = tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let head = format!(
        "POST {route} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n"
    );
    conn.write_all(head.as_bytes()).await.unwrap();
    let mut answer = Vec::new();
    tokio::time::timeout(std::time::Duration::from_secs(seconds), conn.read_to_end(&mut answer))
        .await
        .unwrap_or_else(|_| panic!("{route}: no answer within {seconds} s"))
        .unwrap();
    String::from_utf8_lossy(&answer).to_string()
}

/// With every permit taken, a request is refused with 503 before its body is read: it does not wait for a body that
/// never comes. A request whose body does not arrive gives its permit back with the 408.
#[tokio::test]
async fn busy_costs_nothing() {
    let (model, seen) = listen_slowly(std::time::Duration::from_secs(2), |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let body = json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" });
    let (status, started) = s.start_run(body.clone()).await;
    assert_eq!(status, 202, "{started}");
    // the run holds the only permit once its model is called
    for _ in 0..100 {
        if !seen.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(seen.lock().unwrap().len(), 1, "the model was not called");
    // the body would take `read_timeout_seconds` (2) to be given up: the answer must come before
    for route in ["/run", "/runs"] {
        let text = without_body(s.port, route, 1).await;
        assert!(text.starts_with("HTTP/1.1 503"), "{route}: {text}");
        assert!(text.contains("too_many_runs"), "{route}: {text}");
    }
    let id = started["run"].as_str().unwrap().to_string();
    assert_eq!(s.finished(&id).await["state"], "done");
    // the permit is free: this request takes it, and gives it back with its 408
    let text = without_body(s.port, "/run", 5).await;
    assert!(text.starts_with("HTTP/1.1 408"), "{text}");
    let (status, answer) = s.run(body).await;
    assert_eq!(status, 200, "{answer}");
}
