//! What a model answered when a schema was asked: one place, whatever the provider, reads the text, takes the
//! JSON out of it and checks it against the schema. The providers give the text and the stop reason as they
//! came; what the answer is worth is decided here, and what it does to the run is decided by the caller.

use serde_json::Value;

use super::error::Cut;
use super::response::{ModelResponse, StopReason};

/// What an answer to a schema is.
#[derive(Debug, Clone, PartialEq)]
pub enum Checked {
    /// A JSON value that matches the schema.
    Valid(Value),
    /// The answer is not one JSON value: text, text around the JSON, two values, broken JSON. `why` says
    /// what was found, without what the model wrote.
    NotJson { why: String },
    /// The JSON ends before it closes. `cut` is the limit the output reached, if it reached one; without one,
    /// the model ended by itself and its JSON is broken.
    Incomplete { cut: Option<Cut>, why: String },
    /// The provider told that the model refused to answer.
    Refused,
    /// A JSON value that does not match the schema: the reasons of the validator, which may quote the answer.
    SchemaMismatch { reasons: Vec<String> },
}

/// Reads `response`, the answer to a call that asked for `schema`. The stop reason comes first: a refusal and
/// a cut answer are what they are, whatever the text.
pub fn check(schema: &Value, response: &ModelResponse) -> Checked {
    if response.stop_reason == StopReason::Refused {
        return Checked::Refused;
    }
    let text = response.message.as_deref().unwrap_or_default();
    let cut = match response.stop_reason {
        StopReason::MaxTokens => Some(Cut::MaxTokens),
        StopReason::ContextExceeded => Some(Cut::ContextWindow),
        _ => None,
    };
    if let Some(cut) = cut {
        return Checked::Incomplete {
            cut: Some(cut),
            why: "the output was cut before its end".into(),
        };
    }
    let json = match extract(text) {
        Ok(json) => json,
        Err(why) => return Checked::NotJson { why },
    };
    let value = match serde_json::from_str::<Value>(json) {
        Ok(v) => v,
        Err(e) if e.is_eof() => {
            return Checked::Incomplete {
                cut: None,
                why: format!("the JSON ends before it closes: {e}"),
            };
        }
        Err(e) => {
            return Checked::NotJson {
                why: format!("the answer is not one JSON value: {e}"),
            };
        }
    };
    let reasons = mismatches(schema, &value);
    if reasons.is_empty() {
        Checked::Valid(value)
    } else {
        Checked::SchemaMismatch { reasons }
    }
}

/// The JSON of an answer: the whole text, or the whole content of the one code fence the text is made of
/// (` ``` ` or ` ```json `). Anything else around the JSON is not taken away.
fn extract(text: &str) -> Result<&str, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("the answer is empty".into());
    }
    let Some(rest) = text.strip_prefix("```") else {
        return Ok(text);
    };
    let (opening, body) = rest
        .split_once('\n')
        .ok_or("the answer is a code fence that never ends")?;
    if !matches!(opening.trim(), "" | "json") {
        return Err("the answer is a code fence that is not of JSON".into());
    }
    let body = body
        .strip_suffix("```")
        .ok_or("the answer has text after its code fence")?;
    if body.contains("```") {
        return Err("the answer has more than one code fence".into());
    }
    Ok(body)
}

/// Why `value` does not match `schema`, one reason each, with the place in the JSON; none when it does.
fn mismatches(schema: &Value, value: &Value) -> Vec<String> {
    match jsonschema::validator_for(schema) {
        Ok(validator) => validator
            .iter_errors(value)
            .map(|e| format!("{e} (at `{}`)", e.instance_path()))
            .collect(),
        // the schema is checked when the file is checked; one that cannot be compiled is never accepted
        Err(e) => vec![format!("the schema is not a valid JSON Schema: {e}")],
    }
}
