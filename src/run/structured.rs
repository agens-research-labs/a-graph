//! What an answer to a schema does to the run: the JSON goes on, or the run stops with the error its outcome
//! has. The outcome is decided by `models::structured`; here it becomes the error of the node, and every checked
//! answer is written as `structured.attempt`. What the model wrote is a content: the detail of the error and the
//! `reasons` of the event, never their message.

use serde_json::Value;

use super::Program;
use crate::error::{Error, Path, Phase, Rule};
use crate::logging::{Event, LogLevel};
use crate::models::structured::{Checked, check};
use crate::models::{ModelError, ModelResponse};

impl Program {
    /// The answer of `node`, from model `model`, to a call that asked for `schema`, with its JSON in
    /// `structured`; or the error it stops the run with.
    pub(super) fn structured(
        &self,
        node: &str,
        model: &str,
        schema: &Value,
        mut response: ModelResponse,
    ) -> Result<ModelResponse, Error> {
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(node), rule, msg);
        let who = format!("model `{}` ({})", response.info.model, response.info.provider);
        let said = response.message.clone().unwrap_or_default();
        let checked = check(schema, &response);
        self.attempt_checked(node, model, &checked);
        let not_json = |why: String| {
            fail(
                Rule::SchemaMismatch,
                format!("{who} did not answer the JSON its schema asks for: {why}"),
            )
            .with_detail(said.clone())
        };
        match checked {
            Checked::Valid(value) => {
                (response.structured, response.message) = (Some(value), None);
                // the API is asked once: it is never corrected
                response.info.corrections = Some(0);
                Ok(response)
            }
            Checked::NotJson { why } | Checked::Incomplete { cut: None, why } => Err(not_json(why)),
            Checked::Incomplete { cut: Some(cut), .. } => {
                let cut = ModelError::Truncated {
                    model: response.info.model.clone(),
                    provider: response.info.provider.clone(),
                    cut,
                };
                Err(fail(Rule::ModelFailed, cut.to_string()))
            }
            Checked::Refused => Err(fail(Rule::ModelRefused, format!("{who} refused to answer")).with_detail(said)),
            Checked::SchemaMismatch { reasons } => Err(fail(
                Rule::SchemaMismatch,
                format!("the answer of {who} does not match its schema"),
            )
            .with_detail(format!("{}\nanswer: {said}", reasons.join("; ")))),
        }
    }

    /// `structured.attempt`: one checked answer, a point in the interval of its node. `debug` when it is valid,
    /// `warn` otherwise: never `error`, since the error of the run, if any, comes after it.
    fn attempt_checked(&self, node: &str, model: &str, checked: &Checked) {
        let (outcome, message, reasons) = match checked {
            Checked::Valid(_) => ("valid", "the answer matches its schema".to_string(), None),
            Checked::NotJson { why } => ("not_json", "the answer is not one JSON value".into(), Some(why.clone())),
            Checked::Incomplete { cut: Some(_), why } => (
                "incomplete",
                "the output was cut before its end".into(),
                Some(why.clone()),
            ),
            Checked::Incomplete { cut: None, why } => {
                ("incomplete", "the JSON ends before it closes".into(), Some(why.clone()))
            }
            Checked::Refused => ("refused", "the model refused to answer".into(), None),
            Checked::SchemaMismatch { reasons } => (
                "schema_mismatch",
                "the answer does not match its schema".into(),
                Some(reasons.join("; ")),
            ),
        };
        let level = if outcome == "valid" {
            LogLevel::Debug
        } else {
            LogLevel::Warn
        };
        let mut ev = Event::new(level, "structured.attempt")
            .node(node)
            .field("model", model)
            .field("mechanism", "api")
            .field("attempt", 1)
            .field("max_attempts", 1)
            .field("outcome", outcome)
            .field("message", message);
        if let Some(r) = reasons {
            ev = ev.content("reasons", r);
        }
        self.emit(self.traced(ev));
    }
}
