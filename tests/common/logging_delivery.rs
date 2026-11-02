pub use super::logging::*;

/// A minimal HTTP server: records (headers, body) of every request. Request i is answered with
/// `script[i]` (200 after the script ends); `silent` never answers.
pub async fn mock_server(
    script: Vec<u16>,
    silent: bool,
) -> (String, tokio::sync::mpsc::UnboundedReceiver<(String, String)>) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/ingest", listener.local_addr().unwrap());
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let count = Arc::new(AtomicUsize::new(0));
    let script = Arc::new(script);
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let (tx, count, script) = (tx.clone(), count.clone(), script.clone());
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let n = match sock.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => n,
                    };
                    buf.extend_from_slice(&chunk[..n]);
                    while let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&buf[..end]).to_lowercase();
                        let len = head
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length: "))
                            .map(|v| v.trim().parse::<usize>().unwrap())
                            .unwrap_or(0);
                        if buf.len() < end + 4 + len {
                            break;
                        }
                        let body = String::from_utf8_lossy(&buf[end + 4..end + 4 + len]).into_owned();
                        buf.drain(..end + 4 + len);
                        tx.send((head, body)).unwrap();
                        if silent {
                            continue;
                        }
                        let status = script.get(count.fetch_add(1, Ordering::SeqCst)).copied().unwrap_or(200);
                        sock.write_all(format!("HTTP/1.1 {status} X\r\ncontent-length: 0\r\n\r\n").as_bytes())
                            .await
                            .unwrap();
                    }
                }
            });
        }
    });
    (url, rx)
}

pub async fn drain(rx: &mut tokio::sync::mpsc::UnboundedReceiver<(String, String)>) -> Vec<(String, String)> {
    let mut got = Vec::new();
    while let Ok(Some(r)) = tokio::time::timeout(std::time::Duration::from_millis(300), rx.recv()).await {
        got.push(r);
    }
    got
}

pub fn lines(requests: &[(String, String)]) -> Vec<serde_json::Value> {
    requests
        .iter()
        .flat_map(|(_, b)| b.lines())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

pub fn fast() -> Delivery {
    Delivery {
        timeout: Duration::from_millis(200),
        max_batch: 100,
        retries: 2,
        backoff: Duration::from_millis(10),
    }
}
