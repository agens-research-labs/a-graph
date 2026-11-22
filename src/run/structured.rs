//! What an answer to a schema does to the run: the JSON goes on, the answer is corrected (`schema_prompt` only),
//! or the run stops with the error its outcome has. The outcome is decided by `models::structured`; here it
//! becomes the error of the node, and every checked answer is written as `structured.attempt`. What the model
//! wrote is a content: the detail of the error and the `reasons` of the event, never their message.

use serde_json::Value;

use super::Program;
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::Entity;
use crate::graph::nodes::component::DEFAULT_MAX_ATTEMPTS;
use crate::logging::{Event, LogLevel};
use crate::models::structured::{CORRECTION, Checked, check};
use crate::models::{ChatEntry, ModelError, ModelResponse, Role};

/// Who wrote the schema, which decides what the error of an answer that is not JSON suggests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SchemaOf {
    /// `message.output.schema` of a node: its user may write `schema_prompt` instead.
    Node,
    /// The verdict of a judge or the routing of a react controller: written by the program.
    Program,
}

/// Which mechanism enforces the schema of a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mechanism {
    /// The API of the provider: the answer is asked once.
    Api(SchemaOf),
    /// `schema_prompt`: this is attempt `attempt` of at most `max`.
    Prompt { attempt: u32, max: u32 },
}

/// What follows a checked answer.
pub(super) enum Next {
    /// The answer, with its JSON in `structured`.
    Done(Box<ModelResponse>),
    /// The answer was not one JSON value: these two messages go at the end of the call, and the model is asked
    /// again.
    Correct([ChatEntry; 2]),
}

impl Program {
    /// The schema of the `schema_prompt` of `node`, if it has one.
    pub(super) fn schema_prompt_of(&self, node: Entity) -> Option<Value> {
        self.nodes
            .message()
            .get(node)
            .and_then(|m| m.output.as_ref())
            .and_then(|o| o.schema_prompt.clone())
    }

    /// The `max_attempts` of `node`, or its default.
    pub(super) fn max_attempts_of(&self, node: Entity) -> u32 {
        self.nodes
            .message()
            .get(node)
            .and_then(|m| m.output.as_ref())
            .and_then(|o| o.max_attempts)
            .unwrap_or(DEFAULT_MAX_ATTEMPTS)
    }

    /// The answer of `node`, from model `model`, to a call whose API enforced `schema`, with its JSON in
    /// `structured`; or the error it stops the run with.
    pub(super) fn structured(
        &self,
        node: &str,
        model: &str,
        schema: &Value,
        of: SchemaOf,
        response: ModelResponse,
    ) -> Result<ModelResponse, Error> {
        match self.answer(node, model, schema, Mechanism::Api(of), response)? {
            Next::Done(response) => Ok(*response),
            // the API is asked once: `answer` never corrects it
            Next::Correct(_) => Err(Error::new(
                Phase::Execution,
                Path::node(node),
                Rule::SchemaMismatch,
                "an answer of the API was sent back to be corrected",
            )),
        }
    }

    /// What follows the answer of `node`, from model `model`, to a call that asked for `schema` through
    /// `mechanism`.
    pub(super) fn answer(
        &self,
        node: &str,
        model: &str,
        schema: &Value,
        mechanism: Mechanism,
        mut response: ModelResponse,
    ) -> Result<Next, Error> {
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(node), rule, msg);
        let who = format!("model `{}` ({})", response.info.model, response.info.provider);
        let said = response.message.clone().unwrap_or_default();
        let checked = check(schema, &response);
        let unusable = matches!(checked, Checked::NotJson { .. } | Checked::Incomplete { cut: None, .. });
        let correct = unusable && matches!(mechanism, Mechanism::Prompt { attempt, max } if attempt < max);
        self.attempt_checked(node, model, mechanism, &checked, correct);
        match checked {
            Checked::Valid(value) => {
                (response.structured, response.message) = (Some(value), None);
                response.info.corrections = Some(match mechanism {
                    Mechanism::Api(_) => 0,
                    Mechanism::Prompt { attempt, .. } => attempt - 1,
                });
                Ok(Next::Done(Box::new(response)))
            }
            Checked::NotJson { .. } | Checked::Incomplete { cut: None, .. } if correct => Ok(Next::Correct([
                ChatEntry {
                    role: Role::Assistant,
                    content: said,
                },
                ChatEntry {
                    role: Role::User,
                    content: CORRECTION.to_string(),
                },
            ])),
            Checked::NotJson { why } | Checked::Incomplete { cut: None, why } => Err(match mechanism {
                Mechanism::Prompt { max, .. } => fail(
                    Rule::AttemptLimit,
                    format!("{who} did not answer one JSON value in {max} attempts, the `max_attempts` of the node"),
                )
                .with_detail(format!("{why}\nanswer: {said}")),
                Mechanism::Api(of) => {
                    let hint = match of {
                        SchemaOf::Node => "; use `schema_prompt` with a model whose API does not enforce the schema",
                        SchemaOf::Program => "",
                    };
                    fail(
                        Rule::SchemaMismatch,
                        format!("{who} did not answer the JSON its schema asks for: {why}{hint}"),
                    )
                    .with_detail(said)
                }
            }),
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
    /// `warn` otherwise: never `error`, since the error of the run, if any, comes after it. `correct` says that
    /// the answer is sent back, with `CORRECTION`.
    fn attempt_checked(&self, node: &str, model: &str, mechanism: Mechanism, checked: &Checked, correct: bool) {
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
        let (name, attempt, max) = match mechanism {
            Mechanism::Api(_) => ("api", 1, 1),
            Mechanism::Prompt { attempt, max } => ("prompt", attempt, max),
        };
        let level = if outcome == "valid" {
            LogLevel::Debug
        } else {
            LogLevel::Warn
        };
        let mut ev = Event::new(level, "structured.attempt")
            .node(node)
            .field("model", model)
            .field("mechanism", name)
            .field("attempt", attempt)
            .field("max_attempts", max)
            .field("outcome", outcome)
            .field("message", message);
        if let Some(r) = reasons {
            ev = ev.content("reasons", r);
        }
        if correct {
            ev = ev.content("correction", CORRECTION);
        }
        self.emit(self.traced(ev));
    }
}
