use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::common::service::*;

fn config_path() -> String {
    std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join("service.yml")
        .to_string_lossy()
        .to_string()
}

/// A body of one byte more than 64 MiB is refused with 413, the answer of axum, as it was before the body was read
/// within `read_timeout_seconds`.
#[tokio::test]
async fn body_too_large() {
    let s = start(&config_path()).await;
    let size = 64 * 1024 * 1024 + 1;
    let mut conn = tokio::net::TcpStream::connect(("127.0.0.1", s.port)).await.unwrap();
    let head = format!(
        "POST /run HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {size}\r\n\r\n"
    );
    conn.write_all(head.as_bytes()).await.unwrap();
    // the service may answer and close before the last bytes: a write that fails is not the point of the test
    let _ = conn.write_all(&vec![b' '; size]).await;
    let _ = conn.shutdown().await;
    let mut answer = Vec::new();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), conn.read_to_end(&mut answer)).await;
    let text = String::from_utf8_lossy(&answer);
    assert!(text.starts_with("HTTP/1.1 413"), "{text}");
}
