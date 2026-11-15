//! Calling a utility. It receives ONE JSON object, `{ "text": ... }`, `{ "path": ..., "mime_type": ... }` or
//! `{ "data": "<base64>", "mime_type": ... }`, and answers ONE JSON object, `{ "text": ... }` or
//! `{ "mime_type": ..., "data": "<base64>" }`.
//! A utility that fails stops the run: no model is involved to read the reason, so the reason
//! (the status of its program or of its answer) is the error, and what it wrote is its detail.

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use super::{Utility, UtilityTransport};
use crate::gate;

/// What a utility is given.
#[derive(Debug, Clone, PartialEq)]
pub enum UtilityInput {
    Text(String),
    /// A file, by its path; the type is given when the extension says it.
    File {
        path: String,
        mime_type: Option<String>,
    },
    /// A file in memory, as base64: the same shape a utility answers with. The type is given when it is known.
    Data {
        mime_type: Option<String>,
        data: String,
    },
}

/// What a utility gave back.
#[derive(Debug, Clone, PartialEq)]
pub enum UtilityOutput {
    Text(String),
    /// A file, as base64, with its type.
    File {
        mime_type: String,
        data: String,
    },
}

impl UtilityInput {
    fn to_json(&self) -> Value {
        match self {
            UtilityInput::Text(text) => json!({ "text": text }),
            UtilityInput::File {
                path,
                mime_type: Some(m),
            } => json!({ "path": path, "mime_type": m }),
            UtilityInput::File { path, mime_type: None } => json!({ "path": path }),
            UtilityInput::Data {
                mime_type: Some(m),
                data,
            } => json!({ "data": data, "mime_type": m }),
            UtilityInput::Data { mime_type: None, data } => json!({ "data": data }),
        }
    }
}

/// The type of a file from its extension, for the ones that are well known.
pub fn mime_of(path: &str) -> Option<String> {
    let extension = std::path::Path::new(path).extension()?.to_str()?.to_lowercase();
    let mime = match extension.as_str() {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "html" | "htm" => "text/html",
        "md" => "text/markdown",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        _ => return None,
    };
    Some(mime.to_string())
}

/// A file on disk, as `utility` takes it: a program by its path, a service by its bytes, since a service may not
/// see the disk of a-graph.
pub fn input_of(utility: &Utility, path: &str) -> std::io::Result<UtilityInput> {
    match &utility.transport {
        UtilityTransport::Stdio { .. } => Ok(UtilityInput::File {
            path: path.to_string(),
            mime_type: mime_of(path),
        }),
        UtilityTransport::Http { .. } => Ok(UtilityInput::Data {
            mime_type: mime_of(path),
            data: STANDARD.encode(std::fs::read(path)?),
        }),
    }
}

/// Why a utility failed. `message` never holds what the utility wrote, `detail` does: it is a content,
/// written only where the passport allows contents.
#[derive(Debug, Clone, PartialEq)]
pub struct UtilityFailure {
    pub message: String,
    pub detail: Option<String>,
}

impl UtilityFailure {
    fn plain(message: String) -> Self {
        UtilityFailure { message, detail: None }
    }
}

impl std::fmt::Display for UtilityFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Calls `utility`. `Err` says why it failed, with the id of the utility.
pub async fn call_utility(utility: &Utility, input: &UtilityInput) -> Result<UtilityOutput, UtilityFailure> {
    let arguments = input.to_json();
    let call = async {
        match &utility.transport {
            UtilityTransport::Stdio { command, args } => gate::stdio_call(command, args, &arguments).await,
            UtilityTransport::Http { url, headers } => gate::http_call(url, headers, &arguments).await,
        }
    };
    // when the time is up the call is dropped: a program is killed, a request is abandoned
    let outcome = match tokio::time::timeout(Duration::from_secs(utility.timeout.into()), call).await {
        Ok(outcome) => outcome.map_err(|e| UtilityFailure::plain(format!("utility `{}`: {e}", utility.id)))?,
        Err(_) => {
            return Err(UtilityFailure::plain(format!(
                "utility `{}` did not answer within {} s",
                utility.id, utility.timeout
            )));
        }
    };
    if outcome.is_error {
        let why = match &outcome.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        return Err(match outcome.without_data {
            Some(plain) => UtilityFailure {
                message: format!("utility `{}`: {plain}", utility.id),
                detail: Some(why),
            },
            None => UtilityFailure::plain(format!("utility `{}`: {why}", utility.id)),
        });
    }
    parse(&utility.id, &outcome.result)
}

fn parse(id: &str, answer: &Value) -> Result<UtilityOutput, UtilityFailure> {
    if let Some(text) = answer.get("text").and_then(Value::as_str) {
        return Ok(UtilityOutput::Text(text.to_string()));
    }
    if let (Some(mime_type), Some(data)) = (
        answer.get("mime_type").and_then(Value::as_str),
        answer.get("data").and_then(Value::as_str),
    ) {
        return Ok(UtilityOutput::File {
            mime_type: mime_type.to_string(),
            data: data.to_string(),
        });
    }
    Err(UtilityFailure {
        message: format!("utility `{id}` did not answer with `text` or with `mime_type` and `data`"),
        detail: Some(answer.to_string().chars().take(200).collect::<String>()),
    })
}
