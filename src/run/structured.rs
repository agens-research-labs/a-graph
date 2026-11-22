//! What an answer to a schema does to the run: the JSON goes on, or the run stops with the error its outcome
//! has. The outcome is decided by `models::structured`; here it becomes the error of the node. What the model
//! wrote is the detail of the error, never its message.

use serde_json::Value;

use super::Program;
use crate::error::{Error, Path, Phase, Rule};
use crate::models::structured::{Checked, check};
use crate::models::{ModelError, ModelResponse};

impl Program {
    /// The answer of `node` to a call that asked for `schema`, with its JSON in `structured`; or the error it
    /// stops the run with.
    pub(super) fn structured(
        &self,
        node: &str,
        schema: &Value,
        mut response: ModelResponse,
    ) -> Result<ModelResponse, Error> {
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(node), rule, msg);
        let who = format!("model `{}` ({})", response.info.model, response.info.provider);
        let said = response.message.clone().unwrap_or_default();
        let not_json = |why: String| {
            fail(
                Rule::SchemaMismatch,
                format!("{who} did not answer the JSON its schema asks for: {why}"),
            )
            .with_detail(said.clone())
        };
        match check(schema, &response) {
            Checked::Valid(value) => {
                (response.structured, response.message) = (Some(value), None);
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
}
