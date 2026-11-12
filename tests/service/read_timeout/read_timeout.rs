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

/// With `read_timeout_seconds: 1` a request whose body stops halfway is refused with 408 (`read_timeout`) on both
/// routes that take a body, and its connection is closed with the answer.
#[tokio::test]
async fn read_timeout() {
    let s = start(&config_path()).await;
    for route in ["/run", "/runs"] {
        let mut conn = tokio::net::TcpStream::connect(("127.0.0.1", s.port)).await.unwrap();
        // 8 bytes of the 100 announced, then nothing
        let head = format!(
            "POST {route} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{{\"yml\":\""
        );
        conn.write_all(head.as_bytes()).await.unwrap();
        let mut answer = Vec::new();
        // read to the end: it comes only if the service closes the connection
        tokio::time::timeout(std::time::Duration::from_secs(5), conn.read_to_end(&mut answer))
            .await
            .unwrap_or_else(|_| panic!("{route}: no answer, or the connection stayed open"))
            .unwrap();
        let text = String::from_utf8_lossy(&answer);
        assert!(text.starts_with("HTTP/1.1 408"), "{route}: {text}");
        let body: Value = serde_json::from_str(text.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["errors"][0]["rule"], "read_timeout", "{route}: {body}");
    }
}
