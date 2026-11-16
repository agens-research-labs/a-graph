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
    let (mut reader, mut writer) = conn.split();
    let exchange = async {
        let (_, answer) = tokio::join!(
            async {
                writer.write_all(head.as_bytes()).await.unwrap();
                // The server may refuse before the last bytes; keep reading the refusal concurrently.
                let _ = writer.write_all(&vec![b' '; size]).await;
            },
            async {
                let mut answer = Vec::new();
                loop {
                    let n = reader.read_buf(&mut answer).await.unwrap();
                    if n == 0 || answer.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                answer
            }
        );
        answer
    };
    // Do not half-close the client before consuming the response: that can discard an early 413.
    let answer = tokio::time::timeout(std::time::Duration::from_secs(10), exchange)
        .await
        .expect("the service did not answer the oversized body");
    let text = String::from_utf8_lossy(&answer);
    assert!(text.starts_with("HTTP/1.1 413"), "{text}");
}
