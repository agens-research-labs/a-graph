//! Calling a utility. It receives ONE JSON object, `{ "text": ... }` or `{ "path": ..., "mime_type": ... }`,
//! and answers ONE JSON object, `{ "text": ... }` or `{ "mime_type": ..., "data": "<base64>" }`.
//! A utility that fails stops the run: no model is involved to read the reason, so the reason
//! (the text it wrote on its standard error, or the status of its answer) is the error.

use std::time::Duration;

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

/// Calls `utility`. `Err` says why it failed, with the id of the utility.
pub async fn call_utility(utility: &Utility, input: &UtilityInput) -> Result<UtilityOutput, String> {
    let arguments = input.to_json();
    let call = async {
        match &utility.transport {
            UtilityTransport::Stdio { command, args } => gate::stdio_call(command, args, &arguments).await,
            UtilityTransport::Http { url, headers } => gate::http_call(url, headers, &arguments).await,
        }
    };
    // when the time is up the call is dropped: a program is killed, a request is abandoned
    let outcome = match tokio::time::timeout(Duration::from_secs(utility.timeout.into()), call).await {
        Ok(outcome) => outcome.map_err(|e| format!("utility `{}`: {e}", utility.id))?,
        Err(_) => {
            return Err(format!(
                "utility `{}` did not answer within {} s",
                utility.id, utility.timeout
            ));
        }
    };
    if outcome.is_error {
        let why = match &outcome.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        return Err(format!("utility `{}`: {why}", utility.id));
    }
    parse(&utility.id, &outcome.result)
}

fn parse(id: &str, answer: &Value) -> Result<UtilityOutput, String> {
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
    Err(format!(
        "utility `{id}` did not answer with `text` or with `mime_type` and `data`: {}",
        answer.to_string().chars().take(200).collect::<String>()
    ))
}
