use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, TcpStream};

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

/// Opens a connection from `addr`, sends the headers of a request to `route`, and never sends its body.
async fn headers_only(addr: &str, port: u16, route: &str) -> TcpStream {
    let socket = TcpSocket::new_v4().unwrap();
    socket.bind(format!("{addr}:0").parse().unwrap()).unwrap();
    let mut conn = socket
        .connect(format!("127.0.0.1:{port}").parse().unwrap())
        .await
        .unwrap();
    let head = format!(
        "POST {route} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n"
    );
    conn.write_all(head.as_bytes()).await.unwrap();
    conn
}

/// The whole answer of a connection already open, read until the service closes it, within `seconds`.
async fn answer_of(mut conn: TcpStream, seconds: u64) -> String {
    let mut answer = Vec::new();
    tokio::time::timeout(std::time::Duration::from_secs(seconds), conn.read_to_end(&mut answer))
        .await
        .unwrap_or_else(|_| panic!("no answer within {seconds} s"))
        .unwrap();
    String::from_utf8_lossy(&answer).to_string()
}

/// `conn`'s answer, read within `millis`; `conn` itself if nothing closed it within that time, so its body may
/// still be arriving: it is kept open, not dropped.
async fn peek(mut conn: TcpStream, millis: u64) -> Result<String, TcpStream> {
    let mut answer = Vec::new();
    match tokio::time::timeout(std::time::Duration::from_millis(millis), conn.read_to_end(&mut answer)).await {
        Ok(Ok(_)) => Ok(String::from_utf8_lossy(&answer).to_string()),
        _ => Err(conn),
    }
}

/// Opens `n` requests of `addr` at once (more than `max_reading_per_address`, less than `max_runs`) and sorts
/// them into those still waiting for their body (kept open) and those already refused. The requests reach the
/// service close enough together that which ones win a place is not fixed, only how many: the cap, checked
/// under one lock, lets exactly `max_reading_per_address` through whatever order they arrive in.
async fn open_many(addr: &str, port: u16, route: &str, n: usize) -> (Vec<TcpStream>, Vec<String>) {
    let mut conns = Vec::new();
    for _ in 0..n {
        conns.push(headers_only(addr, port, route).await);
    }
    // a refusal comes at once; one wait for all, so the requests still pending stay far from `read_timeout_seconds`
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let mut pending = Vec::new();
    let mut refused = Vec::new();
    for conn in conns {
        match peek(conn, 100).await {
            Ok(text) => refused.push(text),
            Err(conn) => pending.push(conn),
        }
    }
    (pending, refused)
}

/// With `max_reading_per_address: 2`, of 4 requests of the same address whose body is still arriving, exactly
/// 2 are accepted and 2 are refused with 503, even while `max_runs` (4) has free places. The places are given back
/// with the 408 of a body that never arrives. That another address has places of its own is checked by the unit test
/// `reading_per_address`: macOS has no loopback address but 127.0.0.1.
#[tokio::test]
async fn address_reads_capped() {
    let (model, _seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let body = json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi" });

    let (pending, refused) = open_many("127.0.0.1", s.port, "/run", 4).await;
    assert_eq!(pending.len(), 2, "{refused:?}");
    assert_eq!(refused.len(), 2, "{refused:?}");
    for text in &refused {
        assert!(text.starts_with("HTTP/1.1 503"), "{text}");
        assert!(text.contains("too_many_runs"), "{text}");
    }
    // the two accepted requests give their place back with the 408 of a body that never arrives
    for conn in pending {
        let text = answer_of(conn, 5).await;
        assert!(text.starts_with("HTTP/1.1 408"), "{text}");
    }
    // the address can run again: its places are back
    let (status, answer) = s.run(body).await;
    assert_eq!(status, 200, "{answer}");
}
