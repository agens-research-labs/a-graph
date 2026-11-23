//! How a network output delivers its events: timeout, batches, retries.

use std::time::Duration;

/// How a network output delivers: each POST has a timeout; a network error, a 5xx or a 429 is
/// retried with an exponential backoff; any other answer is final. After the last attempt the
/// batch is dropped and stderr says so.
#[derive(Debug, Clone, Copy)]
pub struct Delivery {
    pub timeout: Duration,
    /// Events per POST, at most.
    pub max_batch: usize,
    /// Attempts after the first one.
    pub retries: u32,
    /// Wait before the first retry; doubles at each one.
    pub backoff: Duration,
}

impl Default for Delivery {
    fn default() -> Self {
        Delivery {
            timeout: Duration::from_secs(10),
            max_batch: 100,
            retries: 3,
            backoff: Duration::from_millis(200),
        }
    }
}

pub(super) async fn deliver(
    client: &reqwest::Client,
    url: &str,
    headers: &reqwest::header::HeaderMap,
    body: String,
    d: &Delivery,
    what: &str,
) {
    let mut last = String::new();
    for attempt in 0..=d.retries {
        match client
            .post(url)
            .headers(headers.clone())
            .body(body.clone())
            .send()
            .await
        {
            Ok(mut r) if r.status().is_success() => {
                if what == "trace" {
                    // A 200 can still reject spans. Never replay a partially accepted batch.
                    // Bound the response and never print remote error messages (untrusted content).
                    let mut bytes = Vec::new();
                    loop {
                        match r.chunk().await {
                            Ok(Some(chunk)) if bytes.len() + chunk.len() <= 65536 => bytes.extend_from_slice(&chunk),
                            Ok(None) => {
                                match trace_reply(&bytes) {
                                    Ok(Some(rejected)) => eprintln!(
                                        "a-graph: trace endpoint reported partial success ({rejected} rejected spans); not retried"
                                    ),
                                    Ok(None) => {}
                                    Err(message) => {
                                        eprintln!("a-graph: {message}; trace acceptance is unknown; not retried")
                                    }
                                }
                                break;
                            }
                            _ => {
                                eprintln!(
                                    "a-graph: cannot read bounded OTLP response; trace acceptance is unknown; not retried"
                                );
                                break;
                            }
                        }
                    }
                }
                return;
            }
            // only a 5xx or a 429 may pass: a 3xx is final too, since redirects are not followed
            Ok(r) if !r.status().is_server_error() && r.status() != reqwest::StatusCode::TOO_MANY_REQUESTS => {
                eprintln!("a-graph: the {what} endpoint refused the data ({})", r.status());
                return;
            }
            Ok(r) => last = format!("the endpoint answered {}", r.status()),
            Err(e) => last = e.to_string(),
        }
        if attempt < d.retries {
            tokio::time::sleep(d.backoff * 2u32.pow(attempt)).await;
        }
    }
    eprintln!("a-graph: {what} data dropped after {} attempts: {last}", d.retries + 1);
}

/// Only the rejected count is safe to report. A populated partialSuccess is never retried,
/// including warning-only responses with zero rejected spans.
fn trace_reply(body: &[u8]) -> Result<Option<u64>, &'static str> {
    let response: serde_json::Value = serde_json::from_slice(body).map_err(|_| "invalid OTLP response")?;
    let object = response.as_object().ok_or("invalid OTLP response")?;
    let Some(partial) = object.get("partialSuccess") else {
        return Ok(None);
    };
    let partial = partial.as_object().ok_or("invalid OTLP partial-success response")?;
    let rejected = match partial.get("rejectedSpans") {
        None => 0,
        Some(serde_json::Value::String(s)) => s.parse::<u64>().map_err(|_| "invalid OTLP rejected span count")?,
        Some(v) => v.as_u64().ok_or("invalid OTLP rejected span count")?,
    };
    let warning = match partial.get("errorMessage") {
        None => false,
        Some(serde_json::Value::String(message)) => !message.is_empty(),
        Some(_) => return Err("invalid OTLP partial-success message"),
    };
    Ok((rejected > 0 || warning).then_some(rejected))
}
