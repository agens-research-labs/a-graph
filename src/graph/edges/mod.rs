//! The edges: the order of execution. The yml list is kept as it is written, with the groups
//! nested. A node can appear more than once: each appearance is a separate execution, and what
//! follows depends on its place in the list, so the order is the list itself.

mod read;

use serde_norway::Value;

use super::nodes::component::StructuredOutput;
use super::nodes::{Entity, Nodes};
use crate::error::Errors;

/// How the branches of a group are executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// A `for` loop over the `fan`.
    Linear,
    /// All together.
    Parallel,
    /// All together, at most `limit` at a time.
    Concurrent,
    /// One batch inference.
    Batch,
}

/// An entry of the list, or a branch of a `fan`.
#[derive(Debug, Clone)]
pub enum Step {
    /// `next: X`
    Node(Entity),
    Group(Group),
    React(React),
}

/// `linear`, `parallel`, `concurrent` or `batch`: the input node `in`, the branches of the `fan`
/// and the output node `out`.
#[derive(Debug, Clone)]
pub struct Group {
    pub pattern: Pattern,
    pub limit: Option<u32>,
    pub r#in: Entity,
    pub fan: Vec<Step>,
    pub out: Option<Entity>,
}

/// A `controller` that dispatches dynamically to its `workers`.
#[derive(Debug, Clone)]
pub struct React {
    pub controller: Entity,
    pub max_iterations: u32,
    /// How the routing output is enforced.
    pub structured_output: StructuredOutput,
    pub workers: Vec<Step>,
}

impl React {
    /// The names the controller calls its workers by: the node, or the `in` node of a group.
    pub fn worker_ids(&self, nodes: &Nodes) -> Vec<String> {
        self.workers
            .iter()
            .map(|w| {
                nodes
                    .id(match w {
                        Step::Node(x) => *x,
                        Step::Group(g) => g.r#in,
                        Step::React(r) => r.controller,
                    })
                    .unwrap_or_default()
                    .to_string()
            })
            .collect()
    }

    /// What the controller answers at every turn: whether it is done, the final answer, or the
    /// worker to call with what it must receive. The program writes it, the user does not. Every
    /// field is required, so a model cannot leave out the one it needs: the ones a turn does not use
    /// are empty (`""`).
    pub fn routing_schema(worker_ids: &[String]) -> serde_json::Value {
        let mut workers: Vec<&str> = worker_ids.iter().map(String::as_str).collect();
        workers.push("");
        serde_json::json!({
            "type": "object",
            "properties": {
                "reasoning": { "type": "string", "description": "Why you chose this step. It is only written in the log." },
                "done": { "type": "boolean", "description": "true when you give the final answer, false when you call a worker." },
                "next_worker": { "type": "string", "enum": workers, "description": "The worker to call when done is false; empty when done is true." },
                "input": { "type": "string", "description": "What the worker receives when done is false; empty when done is true." },
                "answer": { "type": "string", "description": "The final answer when done is true; empty when done is false." }
            },
            "required": ["reasoning", "done", "next_worker", "input", "answer"]
        })
    }
}

#[derive(Debug)]
pub struct Edges {
    start: Entity,
    /// `None` in a graph of one node: the `start` alone.
    end: Option<Entity>,
    /// The entries between `start` and `end`, in the order of the yml.
    steps: Vec<Step>,
}

impl Edges {
    /// Builds from the value of `graph.edges`, already validated by the Loader.
    pub fn build(list: Option<Value>, nodes: &Nodes) -> Result<Self, Errors> {
        read::build(list, nodes)
    }

    pub fn start(&self) -> Entity {
        self.start
    }

    pub fn end(&self) -> Option<Entity> {
        self.end
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }
}
