//! A small HTTP server for the tests: it records every request and answers with a handler.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Notify;

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
    /// Sends the body one byte at a time, this long apart: a server that answers and then drips.
    pub drip: Option<std::time::Duration>,
    /// Bytes sent instead of `body`, which need not be text.
    pub raw: Option<Vec<u8>>,
}

impl Resp {
    pub fn ok(body: impl Into<String>) -> Self {
        Resp {
            status: 200,
            body: body.into(),
            location: None,
            drip: None,
            raw: None,
        }
    }
    pub fn status(status: u16, body: impl Into<String>) -> Self {
        Resp {
            status,
            body: body.into(),
            location: None,
            drip: None,
            raw: None,
        }
    }
    pub fn redirect(status: u16, to: impl Into<String>) -> Self {
        Resp {
            status,
            body: String::new(),
            location: Some(to.into()),
            drip: None,
            raw: None,
        }
    }
    /// A 200 whose body is `bytes`, which need not be text.
    pub fn bytes(bytes: Vec<u8>) -> Self {
        Resp {
            raw: Some(bytes),
            ..Resp::ok("")
        }
    }
    /// The same answer, its body sent one byte every `every`.
    pub fn dripping(self, every: std::time::Duration) -> Self {
        Resp {
            drip: Some(every),
            ..self
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
    start(delay, |_| false, Arc::new(Notify::new()), handler).await
}

/// Like `listen`, and the answer to a request for which `hold(request)` is true is held until the test calls
/// `notify_one` on the `Notify` given back: the test decides when it comes, whatever the speed of the machine.
/// The request is recorded before it is held.
pub async fn listen_held(
    hold: impl Fn(&Req) -> bool + Send + Sync + 'static,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (u16, Arc<Mutex<Vec<Req>>>, Arc<Notify>) {
    let release = Arc::new(Notify::new());
    let (port, seen) = start(|_| std::time::Duration::ZERO, hold, release.clone(), handler).await;
    (port, seen, release)
}

async fn start(
    delay: impl Fn(&Req) -> std::time::Duration + Send + Sync + 'static,
    hold: impl Fn(&Req) -> bool + Send + Sync + 'static,
    release: Arc<Notify>,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (u16, Arc<Mutex<Vec<Req>>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let handler = Arc::new(handler);
    let delay = Arc::new(delay);
    let hold = Arc::new(hold);
    let log = seen.clone();
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let (handler, log, delay, hold, release) = (
                handler.clone(),
                log.clone(),
                delay.clone(),
                hold.clone(),
                release.clone(),
            );
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
                        let held = hold(&req);
                        log.lock().unwrap().push(req);
                        if !wait.is_zero() {
                            tokio::time::sleep(wait).await;
                        }
                        if held {
                            release.notified().await;
                        }
                        let location = resp.location.map(|l| format!("location: {l}\r\n")).unwrap_or_default();
                        let head = format!(
                            "HTTP/1.1 {} X\r\ncontent-type: application/json\r\n{location}content-length: {}\r\n\r\n",
                            resp.status,
                            resp.raw.as_ref().map_or(resp.body.len(), Vec::len),
                        );
                        match (resp.drip, &resp.raw) {
                            // the client may stop reading a body it refuses: then the connection is closed
                            (None, Some(raw)) => {
                                if sock.write_all(head.as_bytes()).await.is_err() || sock.write_all(raw).await.is_err()
                                {
                                    return;
                                }
                            }
                            (None, None) => sock.write_all(format!("{head}{}", resp.body).as_bytes()).await.unwrap(),
                            (Some(every), _) => {
                                // the client may give up halfway: then the connection is closed
                                if sock.write_all(head.as_bytes()).await.is_err() {
                                    return;
                                }
                                for byte in resp.body.as_bytes() {
                                    tokio::time::sleep(every).await;
                                    if sock.write_all(std::slice::from_ref(byte)).await.is_err() {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    });
    (port, seen)
}
