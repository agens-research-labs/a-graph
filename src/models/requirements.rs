//! What a node requires of a model, known from the yml: `Llm::check` answers without the network.

use serde_json::Value;

use super::reasoning::Reasoning;
use crate::resources::{Generate, MediaKind, MediaProvider};

#[derive(Debug, Clone, Default)]
pub struct ModelRequirements {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u32>,
    pub reasoning: Option<Reasoning>,
    /// The output schema, as a JSON object.
    pub schema: Option<Value>,
    /// Types of the media the node sends.
    pub input_media: Vec<MediaKind>,
    /// The media the node wants produced, from their resources.
    pub output_media: Vec<ProducedMedia>,
    /// Tools, MCP servers or skills are bound to the node.
    pub uses_tools: bool,
    /// The node runs in a `batch` block.
    pub batch: bool,
}

/// A media resource with `generate`, as the model has to produce it.
#[derive(Debug, Clone)]
pub struct ProducedMedia {
    pub kind: MediaKind,
    pub generate: Generate,
    pub provider: Option<MediaProvider>,
}
