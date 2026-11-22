//! Running a validation node: the judge reads what its blocks say, with its prompt, and answers
//! the fixed verdict `{ validation, reasons, log }`.

use super::{NodeInput, Program};
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::Entity;
use crate::graph::nodes::component::Validation;

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
        let response = self
            .invoke_model(&id, &prepared.model, llm, &prepared.call)
            .await
            .map_err(|e| fail(Rule::ModelFailed, e.to_string()).with_detail_if(e.said()))?;
        // the verdict matches its schema, or the run stops here: `validation` is a boolean
        let schema = Validation::judge_schema();
        let response = self.structured(&id, &prepared.model, &schema, response)?;
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
