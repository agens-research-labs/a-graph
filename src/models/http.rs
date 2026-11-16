//! What the REST providers share: the client and the way a call is sent and read.

use std::time::Duration;

use serde_json::Value;

use super::error::ModelError;
use super::identity::Identity;

/// A call can take minutes (long outputs, reasoning).
pub(crate) const CALL_TIMEOUT: Duration = Duration::from_secs(600);

pub(crate) fn client() -> Result<reqwest::Client, reqwest::Error> {
    crate::net::client().timeout(CALL_TIMEOUT).build()
}

/// Where a provider writes the parts of its error answers, as JSON pointers tried in order. Each REST provider
/// declares one; `api_error` reads every answer with it.
pub(crate) struct ErrorShape {
    /// The code of the error: the first pointer that holds a text. It is kept only if it is a fixed code
    /// (`fixed_code`), never a text that may quote the request.
    pub(crate) code: &'static [&'static str],
    /// What the provider said: the first pointer that holds a value, as JSON cut at 500 characters when it is not
    /// a text (a list of problems may quote the whole request). It is the `said` of the error, a content; without
    /// it, the first 500 characters of the body.
    pub(crate) message: &'static [&'static str],
    /// Whether the answer is about the credentials, from its status and its body.
    pub(crate) credentials: fn(u16, &str) -> bool,
}

/// 401 and 403 are about the credentials.
pub(crate) fn refused_credentials(status: u16, _body: &str) -> bool {
    status == 401 || status == 403
}

/// The `ModelError` of an error answer, read with the shape of its provider.
pub(crate) fn api_error(who: &Identity, status: u16, body: &str, shape: &ErrorShape) -> ModelError {
    let parsed = serde_json::from_str::<Value>(body).ok();
    // Google wraps its error in a one-element array
    let root = parsed.as_ref().map(|v| match v {
        Value::Array(items) => items.first().unwrap_or(v),
        _ => v,
    });
    let code = root.and_then(|v| shape.code.iter().find_map(|p| v.pointer(p)?.as_str()));
    let said = root
        .and_then(|v| {
            shape.message.iter().find_map(|p| match v.pointer(p)? {
                Value::Null => None,
                Value::String(s) => Some(s.clone()),
                other => Some(other.to_string().chars().take(500).collect()),
            })
        })
        .unwrap_or_else(|| body.chars().take(500).collect());
    if (shape.credentials)(status, body) {
        who.credentials_refused(Some(status), code, Some(said))
    } else {
        who.provider_said(Some(status), code, "", Some(said))
    }
}

/// Sends the request and reads the whole body. A non-2xx answer becomes the `ModelError` that `api_error`
/// reads with the shape of the provider.
pub(crate) async fn send(
    who: &Identity,
    req: reqwest::RequestBuilder,
    errors: &ErrorShape,
) -> Result<String, ModelError> {
    let resp = send_observed(who, req).await?;
    let status = resp.status();
    let text = resp.text().await.map_err(|e| who.network(e))?;
    if status.is_success() {
        Ok(text)
    } else {
        Err(api_error(who, status.as_u16(), &text, errors))
    }
}

/// Like `send`, for an answer that is not text (an audio file, an image).
pub(crate) async fn send_bytes(
    who: &Identity,
    req: reqwest::RequestBuilder,
    errors: &ErrorShape,
) -> Result<Vec<u8>, ModelError> {
    let resp = send_observed(who, req).await?;
    let status = resp.status();
    if status.is_success() {
        return resp.bytes().await.map(|b| b.to_vec()).map_err(|e| who.network(e));
    }
    let text = resp.text().await.map_err(|e| who.network(e))?;
    Err(api_error(who, status.as_u16(), &text, errors))
}

/// Building once gives telemetry the actual method and destination, not an inferred URL.
async fn send_observed(who: &Identity, req: reqwest::RequestBuilder) -> Result<reqwest::Response, ModelError> {
    let (client, request) = req.build_split();
    let request = request.map_err(|e| who.network(e))?;
    let first = super::http_telemetry::started(&request);
    let response = client.execute(request).await.map_err(|e| who.network(e))?;
    super::http_telemetry::responded(first, response.status().as_u16());
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::{ErrorShape, api_error, refused_credentials};
    use crate::models::{Identity, ModelError};

    const SHAPE: ErrorShape = ErrorShape {
        code: &["/error/status", "/error/type"],
        message: &["/error/message", "/detail"],
        credentials: refused_credentials,
    };

    fn read(status: u16, body: &str) -> ModelError {
        api_error(&Identity::new("m", "p"), status, body, &SHAPE)
    }

    /// The pointers are tried in order, inside a one-element array too; a code that is not a text is skipped.
    #[test]
    fn pointers_in_order() {
        let e = read(
            404,
            r#"[{ "error": { "status": 404, "type": "not_found", "message": "no model x" } }]"#,
        );
        assert!(
            matches!(&e, ModelError::Provider { status: Some(404), code: Some(c), said: Some(s), .. } if c == "not_found" && s == "no model x"),
            "{e:?}"
        );
    }

    /// A value that is not a text is kept as JSON, a body that is not JSON as it is; both are cut at 500 characters.
    #[test]
    fn said_without_text() {
        let e = read(422, r#"{ "detail": [{ "input": "x" }] }"#);
        assert_eq!(e.said(), Some(r#"[{"input":"x"}]"#));
        let e = read(
            422,
            &format!(r#"{{ "detail": [{{ "input": "{}" }}] }}"#, "a".repeat(600)),
        );
        assert_eq!(e.said().map(|s| s.chars().count()), Some(500));
        let e = read(500, &"a".repeat(600));
        assert_eq!(e.said().map(str::len), Some(500));
    }

    /// The credentials are decided by the shape: here 401 and 403.
    #[test]
    fn credentials_by_shape() {
        assert!(matches!(read(401, "{}"), ModelError::Credentials { .. }));
        assert!(matches!(read(400, "{}"), ModelError::Provider { .. }));
    }
}
