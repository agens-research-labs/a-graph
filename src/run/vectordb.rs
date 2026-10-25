//! What a node reads from its vector databases: the query is embedded and searched, and the
//! chunks found become the value of the placeholder `{{chunks}}`.

use super::Program;
use super::node::NodeInput;
use crate::logging::{Event, LogLevel};
use crate::error::{Error, Path, Phase, Rule};
use crate::graph::Entity;
use crate::graph::nodes::component::Query;

impl Program {
    /// The chunks of every vectordb of the node, in the order the node lists them, one after the
    /// other with an empty line between; the nearest first inside each database.
    pub(super) async fn read_vectordbs(&self, node: Entity, input: &NodeInput) -> Result<String, Error> {
        let id = self.nodes.id(node).unwrap_or_default().to_string();
        let fail = |rule: Rule, msg: String| Error::new(Phase::Execution, Path::node(&id), rule, msg);
        let Some(external) = self.nodes.external().get(node) else { return Ok(String::new()) };

        let mut chunks: Vec<String> = Vec::new();
        for r in &external.vectordb {
            let began = std::time::Instant::now();
            let db = self.external.vectordb.get(&r.id).ok_or_else(|| fail(Rule::UnknownRef, format!("vectordb `{}` is not built", r.id)))?;
            let query = match &r.query {
                Query::Static(text) => text.clone(),
                Query::UserMessage => input.user_message.clone().ok_or_else(|| fail(Rule::MissingField, format!("the query of vectordb `{}` is the user message and none was given", r.id)))?,
            };
            let query = self.text_through(&id, "vectordb.query", &r.query_utility, query).await?;
            let mut vectors = self.embeddings.embed(&db.embedding, &[query]).await.map_err(|e| fail(Rule::ModelFailed, e.to_string()))?;
            let vector = vectors.pop().ok_or_else(|| fail(Rule::ModelFailed, format!("embedding `{}` returned no vector", db.embedding)))?;
            let found = self.external.vectordb.search(&r.id, &vector).await.map_err(|e| fail(Rule::Io, e))?;
            self.emit(self.traced(Event::new(LogLevel::Debug, "read.vectordb").node(&id).field("id", r.id.clone()).field("chunks", found.len()).field("duration", began.elapsed().as_secs_f64())));
            for chunk in found {
                chunks.push(self.text_through(&id, "vectordb.chunks", &r.chunks_utility, chunk).await?);
            }
        }
        Ok(chunks.join("\n\n"))
    }
}
