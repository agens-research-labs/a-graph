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
            Ok(r) if r.status().is_success() => return,
            Ok(r) if r.status().is_client_error() && r.status() != reqwest::StatusCode::TOO_MANY_REQUESTS => {
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
