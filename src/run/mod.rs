//! The point that connects everything: reads the yml with the Loader, builds the logger, the
//! managers, the nodes and the edges, checks the network permission and sends what happened to the log.

mod batch;
mod events;
mod execute;
mod judge;
mod mcp;
mod memory;
mod node;
mod output;
mod tools;
mod utility;
mod vectordb;

pub use batch::close;
pub use execute::{OutputItem, RunOutput};
pub use node::{Content, Delivery, NodeInput, NodeOutput};

use crate::blackboards::BlackboardManager;
use crate::embeddings::EmbeddingManager;
use crate::error::{Error, Errors, Warning};
use crate::external::External;
use crate::graph::{self, Edges, Nodes};
use crate::loader::{self, Document};
use crate::logging::{Event, Logger, Redaction, Tracer, Verbose};
use crate::models::ModelManager;
use crate::network;
use crate::resources::Resources;
use crate::schema::key;
use crate::utilities::UtilityManager;

/// Everything built from a yml that passed every check.
pub struct Program {
    pub logger: Logger,
    /// The live display of the run (`verbose`).
    pub verbose: Verbose,
    /// The timeline of the run sent to a collector; `None` when there is no `trace` block.
    pub tracer: Option<Tracer>,
    /// Hears every event of the run as it happens (the service uses it to stream them).
    pub observer: Option<std::sync::Arc<dyn Fn(&Event) + Send + Sync>>,
    pub(crate) spans: events::Spans,
    pub models: ModelManager,
    pub embeddings: EmbeddingManager,
    pub utilities: UtilityManager,
    pub resources: Resources,
    pub external: External,
    pub blackboards: BlackboardManager,
    pub nodes: Nodes,
    pub edges: Edges,
    /// What was worth saying while building; already sent to the log.
    pub warnings: Vec<Warning>,
    /// The MCP servers of the run in progress: connected when `run` starts, closed when it ends.
    mcp: std::sync::Mutex<Option<std::sync::Arc<crate::gate::McpPool>>>,
    /// The cancels of the batch jobs the run let go of before they ended, waited for by `close`.
    cancels: batch::Cancels,
}

/// Builds the program from the text of a yml. The errors of the Loader come back alone: nothing
/// is built from a yml that is not coherent. After that the first block is read in order (the
/// logger, the first consumer); every manager then reports ALL its errors, and they are sent to
/// the log as well as returned.
pub fn build(src: &str) -> Result<Program, Errors> {
    let mut doc: Document = loader::load(src)?;

    // the first block: the passport and the logging, before anything else
    let passport = doc.take(key::PASSPORT);
    let observability = doc.take(key::OBSERVABILITY);
    let trace = doc.take(key::TRACE);
    let allowed = network::allowed_hosts(passport.clone())?;
    let mut destinations = network::logging_destinations(observability.clone(), trace.clone())?;
    let verbose = Verbose::build(doc.take(key::VERBOSE), Redaction::from_passport(passport.clone())?)?;
    let tracer = Tracer::build(trace, passport.clone(), Redaction::from_passport(passport.clone())?)?;
    let logger = Logger::build(passport, observability)?;

    // the rest, each block to its manager
    let models = ModelManager::build(doc.take(key::MODELS));
    let embeddings = EmbeddingManager::build(doc.take(key::EMBEDDINGS));
    let utilities = UtilityManager::build(doc.take(key::UTILITIES));
    let resources = Resources::build(doc.take(key::RESOURCES));
    let external = External::build(doc.take(key::EXTERNAL));
    let blackboards = BlackboardManager::build(doc.take(key::BLACKBOARDS));
    let (nodes_value, edges_value) = split_graph(doc.take(key::GRAPH));
    let nodes = Nodes::build(nodes_value);

    let mut errors: Vec<Error> = Vec::new();
    let (models, embeddings, utilities, resources, external, blackboards, nodes) = (
        keep(models, &mut errors),
        keep(embeddings, &mut errors),
        keep(utilities, &mut errors),
        keep(resources, &mut errors),
        keep(external, &mut errors),
        keep(blackboards, &mut errors),
        keep(nodes, &mut errors),
    );
    let (
        Some(models),
        Some(embeddings),
        Some(utilities),
        Some(resources),
        Some(external),
        Some(blackboards),
        Some(nodes),
    ) = (models, embeddings, utilities, resources, external, blackboards, nodes)
    else {
        return Err(report(&logger, errors));
    };
    let Some(edges) = keep(Edges::build(edges_value, &nodes), &mut errors) else {
        return Err(report(&logger, errors));
    };

    let mut broken = graph::interpret(&nodes, &edges, &models, &resources, &external);
    broken.extend(external.memory.check_prompts(&resources.prompts));
    if !broken.is_empty() {
        return Err(report(&logger, broken));
    }

    destinations.extend(models.destinations());
    destinations.extend(embeddings.destinations());
    destinations.extend(utilities.destinations());
    destinations.extend(resources.destinations());
    destinations.extend(external.destinations());
    destinations.extend(blackboards.destinations());
    let refused = network::check(&allowed, destinations);
    if !refused.is_empty() {
        return Err(report(&logger, refused));
    }

    let warnings: Vec<Warning> = resources.warnings().into_iter().cloned().collect();
    for w in &warnings {
        logger.log(&Event::from(w));
    }
    Ok(Program {
        logger,
        verbose,
        tracer,
        observer: None,
        spans: Default::default(),
        models,
        embeddings,
        utilities,
        resources,
        external,
        blackboards,
        nodes,
        edges,
        warnings,
        mcp: Default::default(),
        cancels: Default::default(),
    })
}

/// The value of a manager that built, or its errors added to the list.
fn keep<T>(built: Result<T, Errors>, errors: &mut Vec<Error>) -> Option<T> {
    match built {
        Ok(v) => Some(v),
        Err(e) => {
            errors.extend(e.0);
            None
        }
    }
}

/// The `nodes` and the `edges` out of the `graph` block.
fn split_graph(graph: Option<serde_norway::Value>) -> (Option<serde_norway::Value>, Option<serde_norway::Value>) {
    match graph {
        Some(serde_norway::Value::Mapping(mut g)) => (g.remove(key::NODES), g.remove(key::EDGES)),
        _ => (None, None),
    }
}

/// Sends every error to the log and gives them back as the error of the build.
fn report(logger: &Logger, errors: Vec<Error>) -> Errors {
    for e in &errors {
        logger.log_error(e);
    }
    Errors(errors)
}
