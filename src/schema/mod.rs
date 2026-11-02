//! The description of the yml: the key names, the shape of the values of every block, and what
//! each key is for (`Field::doc`). The Loader validates the yml against it; the managers read
//! their blocks with the same key names (`key`). Its shapes describe the nomenclature and the
//! links between ids, nothing else: the rules about the content of a block belong to its
//! manager, and are told in the descriptions. The JSON Schema (`json`) and the reference in
//! Markdown (`reference`) are generated from it.

pub mod key;

mod json;
mod reference;

pub use json::{json_schema, service_json_schema};
pub use reference::{reference, service_reference};
pub use service::SERVICE;

mod blackboards;
mod edges;
mod embeddings;
mod external;
mod graph;
mod models;
mod observability;
mod passport;
mod resources;
mod service;
mod utilities;

use key::*;

/// A space of ids: a definition (`Shape::Id`) and a citation (`Shape::Ref`) of the same kind
/// are linked by the Loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Model,
    Embedding,
    Utility,
    Prompt,
    Tool,
    Mcp,
    Skill,
    Media,
    Vectordb,
    Search,
    Memory,
    Blackboard,
    Node,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Model => "model",
            Kind::Embedding => "embedding",
            Kind::Utility => "utility",
            Kind::Prompt => "prompt",
            Kind::Tool => "tool",
            Kind::Mcp => "mcp server",
            Kind::Skill => "skill",
            Kind::Media => "media",
            Kind::Vectordb => "vectordb",
            Kind::Search => "search",
            Kind::Memory => "memory",
            Kind::Blackboard => "blackboard",
            Kind::Node => "node",
        }
    }
}

/// The shape of a value.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Text,
    /// A whole number.
    Int,
    /// Any number.
    Number,
    Bool,
    /// One of the listed words.
    Words(&'static [&'static str]),
    /// A text that defines an id of the kind.
    Id(Kind),
    /// A text that cites an id of the kind.
    Ref(Kind),
    List(&'static Shape),
    /// A map with these keys and no others.
    Map(&'static [Field]),
    /// A map with any keys, all with the same shape of value.
    MapOf(&'static Shape),
    /// A map whose key `by` chooses which fields it has, besides the common ones. `by_doc`
    /// describes that key.
    Tagged {
        by: &'static str,
        by_doc: &'static str,
        common: &'static [Field],
        variants: &'static [Variant],
    },
    /// One of several forms.
    Either(&'static [Shape]),
    /// A named shape, reached through a function: one that contains itself (a group inside a
    /// group), or that is used in several places. The name is its definition in the JSON Schema.
    Lazy {
        name: &'static str,
        doc: &'static str,
        get: fn() -> &'static Shape,
    },
    /// Free content, accepted as written (a JSON Schema, for example).
    Any,
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub name: &'static str,
    pub shape: Shape,
    pub required: bool,
    /// What the key is for, in the reference generated from this description. Empty until written.
    pub doc: &'static str,
}

impl Field {
    pub const fn req(name: &'static str, shape: Shape) -> Self {
        Field {
            name,
            shape,
            required: true,
            doc: "",
        }
    }
    pub const fn opt(name: &'static str, shape: Shape) -> Self {
        Field {
            name,
            shape,
            required: false,
            doc: "",
        }
    }
    pub const fn doc(mut self, doc: &'static str) -> Self {
        self.doc = doc;
        self
    }
}

/// One value of the key of a `Tagged` map, with the fields it adds.
#[derive(Debug, Clone, Copy)]
pub struct Variant {
    pub name: &'static str,
    pub fields: &'static [Field],
    pub doc: &'static str,
}

impl Variant {
    pub const fn new(name: &'static str, fields: &'static [Field], doc: &'static str) -> Self {
        Variant { name, fields, doc }
    }
}

/// `{ env: VAR_NAME }`: secrets are never written in the yml.
pub const SECRET: Shape =
    Shape::Map(&[Field::req(ENV, Shape::Text).doc("The name of the environment variable that holds the value.")]);

/// The whole document.
pub const DOCUMENT: &[Field] = &[
    Field::req(GRAPH_SPEC, Shape::Words(&["1.0"])).doc("The version of the format of this file."),
    Field::req(PASSPORT, passport::PASSPORT).doc(
        "The identity of the graph, in the ADL 0.3.0 format (https://www.adl-spec.org/), with the hosts it may reach.",
    ),
    Field::req(OBSERVABILITY, observability::OBSERVABILITY)
        .doc("The log: structured records of what the program does, for machines."),
    Field::opt(TRACE, observability::TRACE)
        .doc("The timeline of the run, sent to a collector when the run ends. Without this block there is no trace."),
    Field::req(VERBOSE, observability::VERBOSE)
        .doc("The live display of the run, for a person, on the standard error."),
    Field::req(MODELS, models::MODELS).doc("The language models the nodes call."),
    Field::opt(EMBEDDINGS, embeddings::EMBEDDINGS)
        .doc("The models that turn a text into a vector, for the vector databases and the memories."),
    Field::opt(UTILITIES, utilities::UTILITIES)
        .doc("Programs and services that the program, never the model, calls to change the data of a node."),
    Field::opt(RESOURCES, resources::RESOURCES).doc("What the nodes use: prompts, tools, MCP servers, media, skills."),
    Field::opt(EXTERNAL, external::EXTERNAL)
        .doc("The sources outside the program: vector databases, search engines, memories."),
    Field::opt(BLACKBOARDS, blackboards::BLACKBOARDS).doc("Markdown texts shared by the nodes."),
    Field::req(GRAPH, graph::GRAPH).doc("The nodes and the edges that connect them."),
];
