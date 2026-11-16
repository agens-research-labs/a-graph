//! The components of a node: pure data, one type for each block a node can have. A node is an
//! entity (its id) that carries some of them; a validation node is the one that carries `Validation`
//! besides the components it reads with.

use serde_json::Value as Json;

use crate::blackboards::Place;
use crate::models::Reasoning;

/// The model a node is bound to, and the parameters of its calls.
#[derive(Debug, Clone)]
pub struct Model {
    pub id: String,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    /// The only budget control: it lives on the node, not on the model.
    pub max_tokens: Option<u32>,
    pub reasoning: Option<Reasoning>,
}

/// `query: user_message`, or a fixed query written in the yml.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    UserMessage,
    Static(String),
}

#[derive(Debug, Clone)]
pub struct Retrieval {
    pub id: String,
    pub query: Query,
    /// Utilities the question goes through before the search.
    pub query_utility: Vec<String>,
    /// Utilities each chunk goes through before the prompt.
    pub chunks_utility: Vec<String>,
}

/// Which roles of the chat are read or written.
#[derive(Debug, Clone, Copy)]
pub struct Roles {
    pub user: bool,
    pub assistant: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryUse {
    pub id: String,
    pub read: Roles,
    pub write: Roles,
    /// Utilities what is read (each message and the summary) goes through.
    pub read_utility: Vec<String>,
    /// Utilities what is saved goes through, after `message.output.utility`.
    pub write_utility: Vec<String>,
}

/// What the node takes from outside the process.
#[derive(Debug, Clone, Default)]
pub struct External {
    pub vectordb: Vec<Retrieval>,
    pub search: Vec<Retrieval>,
    pub memory: Vec<MemoryUse>,
}

#[derive(Debug, Clone, Default)]
pub struct Resources {
    pub prompt: Option<String>,
    /// Ids of `resources.tools`: the cap of calls belongs to the tool.
    pub tools: Vec<String>,
    /// Ids of `resources.mcp_servers`: the caps belong to the server.
    pub mcp_servers: Vec<String>,
    pub skills: Vec<String>,
    /// The most rounds of tool requests of the model in one run of the node; `None` is `DEFAULT_MAX_ROUNDS`.
    pub max_rounds: Option<u32>,
}

/// The default of `resources.max_rounds`.
pub const DEFAULT_MAX_ROUNDS: u32 = 20;

#[derive(Debug, Clone)]
pub struct BlackboardWrite {
    pub enabled: bool,
    pub place: Place,
    /// Utilities the text goes through before it is written, after `message.output.utility`.
    pub utility: Vec<String>,
}

/// A node manages exactly ONE blackboard.
#[derive(Debug, Clone)]
pub struct Blackboard {
    pub id: String,
    pub read: bool,
    pub write: Option<BlackboardWrite>,
    pub cleanup: bool,
}

/// What is not text.
#[derive(Debug, Clone, Default)]
pub struct Media {
    pub input: Vec<MediaIn>,
    pub output: Vec<MediaOut>,
}

/// Where an input media comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InFrom {
    /// A file declared in `resources.media`.
    Resources,
    /// A media produced by the node connected before.
    Node,
}

/// A media the node reads.
#[derive(Debug, Clone)]
pub struct MediaIn {
    pub id: String,
    pub from: InFrom,
    /// Ids of `utilities` the media goes through before the node uses it, in order.
    pub utility: Vec<String>,
}

/// Where an output media comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutFrom {
    /// The model makes it (an image, a voice).
    Model,
    /// A utility makes it from the text of the node.
    Message,
}

/// A media the node produces, and where it goes.
#[derive(Debug, Clone)]
pub struct MediaOut {
    pub id: String,
    pub from: OutFrom,
    pub utility: Vec<String>,
    /// Kept in the final output.
    pub assistant: bool,
    /// Passed to the connected node.
    pub node: bool,
}

/// The text.
#[derive(Debug, Clone, Default)]
pub struct Message {
    pub input: Option<MessageInput>,
    pub output: Option<MessageOutput>,
    /// Ids of `utilities` the messages the node takes go through, in order.
    pub input_utility: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MessageInput {
    pub user_message: bool,
    pub agent_message: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MessageOutput {
    pub assistant_message: bool,
    pub node: bool,
    pub reasoning: bool,
    /// A JSON Schema as a JSON object; absent = raw text.
    pub schema: Option<Json>,
    /// Ids of `utilities` the text produced goes through, once, before every destination.
    pub utility: Vec<String>,
}

/// What happens when the judge says no.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnError {
    /// The run ends and goes straight to the final output.
    Stop,
    /// The run goes on: the problem is the `agent_message` of the next node.
    Node,
}

/// Marks a node as a validator: a node that reads and judges. The rest of the node (model, prompt,
/// what it reads) is in the ordinary components; the answer of the judge is fixed.
#[derive(Debug, Clone)]
pub struct Validation {
    pub on_error: OnError,
}

impl Validation {
    /// The fixed answer of the judge, as a JSON Schema: every field is required.
    pub fn judge_schema() -> Json {
        serde_json::json!({
            "type": "object",
            "properties": {
                "validation": { "type": "boolean" },
                "reasons": { "type": "string" },
                "log": { "type": "string" }
            },
            "required": ["validation", "reasons", "log"],
            "additionalProperties": false
        })
    }
}
