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
}

impl Resp {
    pub fn ok(body: impl Into<String>) -> Self {
        Resp { status: 200, body: body.into() }
    }
    pub fn status(status: u16, body: impl Into<String>) -> Self {
        Resp { status, body: body.into() }
    }
}

/// Starts the server; returns its base URL and the recorded requests.
pub async fn serve(handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    serve_on("127.0.0.1:0", handler).await
}

/// Like `serve`, on the port the yml of the test names.
pub async fn serve_at(port: u16, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    serve_on(&format!("127.0.0.1:{port}"), handler).await
}

/// Like `serve_at`, and every answer waits `delay` first (without blocking a thread: a handler that sleeps
/// would hold up the other tasks of the test).
pub async fn serve_at_slowly(port: u16, delay: std::time::Duration, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    serve_with(&format!("127.0.0.1:{port}"), move |_| delay, handler).await
}

/// Like `serve_at`, and every answer waits `delay(request)` first, without blocking a thread.
pub async fn serve_at_delayed(port: u16, delay: impl Fn(&Req) -> std::time::Duration + Send + Sync + 'static, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    serve_with(&format!("127.0.0.1:{port}"), delay, handler).await
}

async fn serve_on(addr: &str, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    serve_with(addr, |_| std::time::Duration::ZERO, handler).await
}

async fn serve_with(addr: &str, delay: impl Fn(&Req) -> std::time::Duration + Send + Sync + 'static, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (String, Arc<Mutex<Vec<Req>>>) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| panic!("cannot listen on {addr} (a port named by a fixture is in use?): {e}"));
    let base = format!("http://{}", listener.local_addr().unwrap());
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
                        let len = lower.lines().find_map(|l| l.strip_prefix("content-length: ")).map(|v| v.trim().parse::<usize>().unwrap()).unwrap_or(0);
                        if buf.len() < end + 4 + len {
                            break;
                        }
                        let body = String::from_utf8_lossy(&buf[end + 4..end + 4 + len]).into_owned();
                        buf.drain(..end + 4 + len);
                        let mut first = head.lines().next().unwrap().split(' ');
                        let req = Req { method: first.next().unwrap().into(), path: first.next().unwrap().into(), headers: lower, body };
                        let resp = handler(&req);
                        let wait = delay(&req);
                        log.lock().unwrap().push(req);
                        if !wait.is_zero() {
                            tokio::time::sleep(wait).await;
                        }
                        let out = format!("HTTP/1.1 {} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}", resp.status, resp.body.len(), resp.body);
                        sock.write_all(out.as_bytes()).await.unwrap();
                    }
                }
            });
        }
    });
    (base, seen)
}
