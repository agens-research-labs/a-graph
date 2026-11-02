//! A small HTTP server for the tests: it records every request and answers with a handler.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct Req {
    pub method: String,
    pub path: String,
    /// Header lines, lowercase.
    pub headers: String,
    pub body: String,
}

pub struct Resp {
    pub status: u16,
    pub body: String,
    /// The `location` header, for a redirect.
    pub location: Option<String>,
}

impl Resp {
    pub fn ok(body: impl Into<String>) -> Self {
        Resp {
            status: 200,
            body: body.into(),
            location: None,
        }
    }
    pub fn status(status: u16, body: impl Into<String>) -> Self {
        Resp {
            status,
            body: body.into(),
            location: None,
        }
    }
    pub fn redirect(status: u16, to: impl Into<String>) -> Self {
        Resp {
            status,
            body: String::new(),
            location: Some(to.into()),
        }
    }
}

/// Starts the server on a free port; returns its base URL and the recorded requests.
pub async fn serve(handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    let (port, seen) = listen(handler).await;
    (format!("http://127.0.0.1:{port}"), seen)
}

/// Like `serve`, and gives the port instead of the URL: the port a fixture names with `{{port:<name>}}`.
pub async fn listen(handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (u16, Arc<Mutex<Vec<Req>>>) {
    listen_slowly(std::time::Duration::ZERO, handler).await
}

/// Like `listen`, and every answer waits `delay` first (without blocking a thread: a handler that sleeps
/// would hold up the other tasks of the test).
pub async fn listen_slowly(
    delay: std::time::Duration,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (u16, Arc<Mutex<Vec<Req>>>) {
    listen_delayed(move |_| delay, handler).await
}

/// Like `listen`, and every answer waits `delay(request)` first, without blocking a thread.
pub async fn listen_delayed(
    delay: impl Fn(&Req) -> std::time::Duration + Send + Sync + 'static,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (u16, Arc<Mutex<Vec<Req>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let handler = Arc::new(handler);
    let delay = Arc::new(delay);
    let log = seen.clone();
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let (handler, log, delay) = (handler.clone(), log.clone(), delay.clone());
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 8192];
                loop {
                    let n = match sock.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => n,
                    };
                    buf.extend_from_slice(&chunk[..n]);
                    while let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&buf[..end]).to_string();
                        let lower = head.to_lowercase();
                        let len = lower
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length: "))
                            .map(|v| v.trim().parse::<usize>().unwrap())
                            .unwrap_or(0);
                        if buf.len() < end + 4 + len {
                            break;
                        }
                        let body = String::from_utf8_lossy(&buf[end + 4..end + 4 + len]).into_owned();
                        buf.drain(..end + 4 + len);
                        let mut first = head.lines().next().unwrap().split(' ');
                        let req = Req {
                            method: first.next().unwrap().into(),
                            path: first.next().unwrap().into(),
                            headers: lower,
                            body,
                        };
                        let resp = handler(&req);
                        let wait = delay(&req);
                        log.lock().unwrap().push(req);
                        if !wait.is_zero() {
                            tokio::time::sleep(wait).await;
                        }
                        let location = resp.location.map(|l| format!("location: {l}\r\n")).unwrap_or_default();
                        let out = format!(
                            "HTTP/1.1 {} X\r\ncontent-type: application/json\r\n{location}content-length: {}\r\n\r\n{}",
                            resp.status,
                            resp.body.len(),
                            resp.body
                        );
                        sock.write_all(out.as_bytes()).await.unwrap();
                    }
                }
            });
        }
    });
    (port, seen)
}
