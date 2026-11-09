//! Running a validation node: the judge reads what its blocks say, with its prompt, and answers
//! the fixed verdict `{ validation, reasons, log }`.

use super::{NodeInput, Program};
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::Entity;

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
        let response = llm
            .invoke(&prepared.call)
            .await
            .map_err(|e| fail(Rule::ModelFailed, e.to_string()))?;

        self.model_called(&id, &prepared.model, &prepared.call, &response);
        let verdict = response
            .structured
            .clone()
            .filter(serde_json::Value::is_object)
            .ok_or_else(|| fail(Rule::BadValue, "the judge did not answer with the verdict".into()))?;
        let approved = verdict["validation"].as_bool().ok_or_else(|| {
            fail(Rule::BadValue, "the verdict of the judge has no `validation`".into()).with_detail(verdict.to_string())
        })?;
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
