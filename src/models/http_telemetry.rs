//! Bounded, invocation-local metadata from the shared REST transport. No URLs, headers or bodies
//! are retained. Direct provider calls without an observation scope keep their existing behavior.

use std::cell::RefCell;
use std::future::Future;

use serde_json::{Value, json};

#[derive(Clone, Default)]
pub(crate) struct HttpMetadata {
    first: Option<RequestMetadata>,
    requests: u64,
}

#[derive(Clone)]
struct RequestMetadata {
    server: Option<String>,
    port: Option<u16>,
    scheme: String,
    method: String,
    status: Option<u16>,
}

tokio::task_local! {
    static HTTP: RefCell<HttpMetadata>;
}

/// Scope follows the invocation future across awaits, but never leaks to parallel sibling calls.
pub(crate) async fn observe<T>(future: impl Future<Output = T>) -> (T, HttpMetadata) {
    HTTP.scope(RefCell::new(HttpMetadata::default()), async {
        let result = future.await;
        (result, HTTP.with(|metadata| metadata.borrow().clone()))
    })
    .await
}

/// Record only the first REST request of a logical invocation, plus a bounded scalar count.
pub(crate) fn started(request: &reqwest::Request) -> bool {
    HTTP.try_with(|metadata| {
        let mut metadata = metadata.borrow_mut();
        metadata.requests = metadata.requests.saturating_add(1);
        if metadata.first.is_some() {
            return false;
        }
        let url = request.url();
        metadata.first = Some(RequestMetadata {
            server: url
                .host_str()
                .map(|h| h.trim_start_matches('[').trim_end_matches(']').to_string()),
            port: url.port_or_known_default(),
            scheme: url.scheme().to_string(),
            method: request.method().to_string(),
            status: None,
        });
        true
    })
    .unwrap_or(false)
}

pub(crate) fn responded(first: bool, status: u16) {
    if first {
        let _ = HTTP.try_with(|metadata| {
            if let Some(request) = &mut metadata.borrow_mut().first {
                request.status = Some(status);
            }
        });
    }
}

impl HttpMetadata {
    pub(crate) fn fields(self) -> Vec<(String, Value)> {
        let mut fields = Vec::new();
        if let Some(request) = self.first {
            if let Some(server) = request.server {
                fields.push(("server.address".into(), json!(server)));
            }
            if let Some(port) = request.port {
                fields.push(("server.port".into(), json!(port)));
            }
            fields.push(("url.scheme".into(), json!(request.scheme)));
            fields.push(("http.request.method".into(), json!(request.method)));
            if let Some(status) = request.status {
                fields.push(("http.response.status_code".into(), json!(status)));
            }
        }
        if self.requests > 1 {
            fields.push(("http_request_count".into(), json!(self.requests)));
        }
        fields
    }
}
