//! The managers of the `resources` block: things a node uses directly, no external service
//! involved. Each one is built from its own list, already validated by the Loader, and checks
//! the content of its items.

mod common;
mod mcp;
mod media;
mod prompts;
mod skills;
mod source;
mod tools;

pub use mcp::*;
pub use media::*;
pub use prompts::*;
pub use skills::*;
pub use source::Source;
pub use tools::*;

use serde_norway::Value;

use crate::error::{Errors, Warning};
use crate::network::Destination;
use crate::schema::key;

/// The five managers, built from the value of the `resources` block.
#[derive(Debug, Default)]
pub struct Resources {
    pub prompts: PromptsManager,
    pub tools: ToolsManager,
    pub mcp: McpManager,
    pub media: MediaManager,
    pub skills: SkillsManager,
}

impl Resources {
    /// Every network destination of the resources, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        [
            self.prompts.destinations(),
            self.tools.destinations(),
            self.mcp.destinations(),
            self.media.destinations(),
        ]
        .concat()
    }

    /// What the managers found worth saying while they built.
    pub fn warnings(&self) -> Vec<&Warning> {
        self.skills.warnings().iter().collect()
    }

    /// Builds all five. On failure returns the errors of every manager, not only the first.
    pub fn build(block: Option<Value>) -> Result<Self, Errors> {
        let mut block = match block {
            Some(Value::Mapping(m)) => m,
            _ => Default::default(),
        };
        let prompts = PromptsManager::build(block.remove(key::PROMPTS));
        let tools = ToolsManager::build(block.remove(key::TOOLS));
        let mcp = McpManager::build(block.remove(key::MCP_SERVERS));
        let media = MediaManager::build(block.remove(key::MEDIA));
        let skills = SkillsManager::build(block.remove(key::SKILLS));

        match (prompts, tools, mcp, media, skills) {
            (Ok(prompts), Ok(tools), Ok(mcp), Ok(media), Ok(skills)) => Ok(Resources {
                prompts,
                tools,
                mcp,
                media,
                skills,
            }),
            (a, b, c, d, e) => {
                let mut all = Vec::new();
                for r in [a.err(), b.err(), c.err(), d.err(), e.err()].into_iter().flatten() {
                    all.extend(r.0);
                }
                Err(Errors(all))
            }
        }
    }
}
