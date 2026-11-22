//! Running a validation node: the judge reads what its blocks say, with its prompt, and answers
//! the fixed verdict `{ validation, reasons, log }`.

use super::{NodeInput, Program};
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::Entity;
use crate::graph::nodes::component::{StructuredOutput, Validation};

/// What the judge decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Verdict {
    Approved,
    Refused { reasons: String, log: String },
}

impl Program {
    /// Calls the judge of a validation node.
    pub(super) async fn judge(&self, node: Entity, input: &NodeInput) -> Result<Verdict, Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        let prepared = self.prepare_node(node, input).await?;
        let llm = self
            .models
            .get(&prepared.model)
            .ok_or_else(|| fail(Rule::UnknownRef, format!("model `{}` is not built", prepared.model)))?;
        // the verdict matches its schema, or the run stops here: `validation` is a boolean
        let mechanism = self
            .nodes
            .validation()
            .get(node)
            .map_or(StructuredOutput::Api, |v| v.structured_output);
        let mut call = prepared.call;
        let schema = Validation::judge_schema();
        let response = self
            .ask(&id, &prepared.model, llm, &mut call, &schema, mechanism)
            .await?;
        let verdict = response.structured.clone().unwrap_or_default();
        let approved = verdict["validation"] == serde_json::Value::Bool(true);
        self.node_finished(node, &response, Some(if approved { "approved" } else { "refused" }));
        if approved {
            return Ok(Verdict::Approved);
        }
        let text = |name: &str| verdict[name].as_str().unwrap_or_default().to_string();
        Ok(Verdict::Refused {
            reasons: text("reasons"),
            log: text("log"),
        })
    }
}
