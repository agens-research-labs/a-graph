//! What the REST providers share: the client and the way a call is sent and read.

use std::time::Duration;

use super::error::ModelError;
use super::identity::Identity;

/// A call can take minutes (long outputs, reasoning).
const CALL_TIMEOUT: Duration = Duration::from_secs(600);

pub(crate) fn client() -> Result<reqwest::Client, reqwest::Error> {
    crate::net::client().timeout(CALL_TIMEOUT).build()
}

/// Sends the request and reads the whole body. A non-2xx answer becomes the `ModelError` that
/// `on_error(status, body)` builds: each provider knows its own error format.
pub(crate) async fn send(
    who: &Identity,
    req: reqwest::RequestBuilder,
    on_error: impl Fn(u16, &str) -> ModelError,
) -> Result<String, ModelError> {
    let resp = req.send().await.map_err(|e| who.network(e))?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| who.network(e))?;
    if status.is_success() {
        Ok(text)
    } else {
        Err(on_error(status.as_u16(), &text))
    }
}

/// Like `send`, for an answer that is not text (an audio file, an image).
pub(crate) async fn send_bytes(
    who: &Identity,
    req: reqwest::RequestBuilder,
    on_error: impl Fn(u16, &str) -> ModelError,
) -> Result<Vec<u8>, ModelError> {
    let resp = req.send().await.map_err(|e| who.network(e))?;
    let status = resp.status();
    if status.is_success() {
        return resp.bytes().await.map(|b| b.to_vec()).map_err(|e| who.network(e));
    }
    let text = resp.text().await.map_err(|e| who.network(e))?;
    Err(on_error(status.as_u16(), &text))
}
