//! The MCP servers of a run: connected when the run starts, only the ones a node uses, and closed
//! when it ends. Building the program never starts them.

use std::collections::HashSet;
use std::sync::Arc;

use super::Program;
use crate::error::{Error, Path, Phase, Rule};
use crate::gate::McpPool;
use crate::logging::{Event, LogLevel};
use crate::schema::key;

impl Program {
    /// Connects the servers some node uses and checks that no node would see two tools with the
    /// same name, which the model could not tell apart.
    pub(super) async fn connect_mcp(&self) -> Result<(), Error> {
        let used: Vec<String> = {
            let mut seen = HashSet::new();
            self.nodes
                .entities()
                .filter_map(|e| self.nodes.resources().get(e))
                .flat_map(|r| r.mcp_servers.iter())
                .filter(|id| seen.insert(id.as_str().to_string()))
                .cloned()
                .collect()
        };
        if used.is_empty() {
            return Ok(());
        }
        let at = Path::root().key(key::RESOURCES).key(key::MCP_SERVERS);
        let servers: Vec<_> = used
            .iter()
            .filter_map(|id| self.resources.mcp.get(id))
            .cloned()
            .collect();
        let pool = McpPool::connect(&servers)
            .await
            .map_err(|e| Error::new(Phase::Execution, at.clone(), Rule::Io, e))?;

        for e in self.nodes.entities() {
            let Some(r) = self.nodes.resources().get(e) else {
                continue;
            };
            let id = self.nodes.id(e).unwrap_or_default();
            let mut names: HashSet<String> = r.tools.iter().cloned().collect();
            if !r.skills.is_empty() {
                names.insert(crate::graph::ACTIVATE_SKILL.to_string());
                names.insert(crate::graph::READ_SKILL_FILE.to_string());
                names.insert(crate::graph::RUN_SKILL_SCRIPT.to_string());
            }
            for server in &r.mcp_servers {
                for t in pool.tools(server) {
                    if !names.insert(t.name.clone()) {
                        let msg = format!(
                            "node `{id}` would have two tools called `{}` (server `{server}` is one of the sources): the model could not tell them apart",
                            t.name
                        );
                        pool.close().await;
                        return Err(Error::new(Phase::Execution, Path::node(id), Rule::DuplicateId, msg));
                    }
                }
            }
        }
        // a schema that cannot be compiled is not enforced: the user is told, once per tool
        for server in &used {
            for t in pool.tools(server) {
                if let Err(e) = jsonschema::validator_for(&t.parameters) {
                    self.emit(
                        Event::new(LogLevel::Warn, "tool_schema_ignored")
                            .field("phase", Phase::Execution.to_string())
                            .field("path", Path::root().key(key::RESOURCES).key(key::MCP_SERVERS).to_string())
                            .field("message", format!("MCP server `{server}`: the schema of the tool `{}` is not valid ({e}), its arguments are not checked", t.name)),
                    );
                }
            }
        }
        *self.mcp.lock().unwrap() = Some(Arc::new(pool));
        Ok(())
    }

    /// Closes the servers of the run that ends.
    pub(super) async fn close_mcp(&self) {
        let pool = self.mcp.lock().unwrap().take();
        if let Some(pool) = pool.and_then(|p| Arc::try_unwrap(p).ok()) {
            pool.close().await;
        }
    }

    /// The pool of the run in progress.
    pub(super) fn mcp_pool(&self) -> Option<Arc<McpPool>> {
        self.mcp.lock().unwrap().clone()
    }
}
