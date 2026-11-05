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
use crate::error::{Error, Errors, Path, Warning};
use crate::external::External;
use crate::graph::{self, Edges, Nodes};
use crate::loader::{self, Document};
use crate::logging::{Event, Logger, Redaction, Tracer, Verbose};
use crate::models::ModelManager;
use crate::network;
use crate::resources::Resources;
use crate::schema::key;
use crate::secrets::{self, AllowedEnv};
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
/// logger, the first consumer); a log url whose host is not allowed comes back alone, before the
/// logger is built: the logger sends from its build on, so that url is never called. Then, each
/// step only when the one before found nothing, and with ALL the errors of the step sent to the
/// log as well as returned:
///
/// 1. every manager reads its block, and no environment variable is read;
/// 2. every destination is checked against the passport, and, in the service, every environment
///    variable against `allowed_env` (see `secrets`);
/// 3. the environment variables are read: the models are made ready to be called, and so is the
///    tracer;
/// 4. the graph is checked against what the models and the other managers can do.
///
/// From the command line every environment variable may be used: the one who writes the graph
/// runs it, with their own variables.
pub fn build(src: &str) -> Result<Program, Errors> {
    build_with(src, None)
}

/// Like `build`, for a graph sent to the service: every environment variable the graph names, and
/// every host its value goes to, must be in `allowed_env`. The variables of the log are checked
/// with its url, before it is built; the others with the destinations. A variable refused is
/// never read, so whether it is set is never told.
pub fn build_with_env(src: &str, allowed_env: &AllowedEnv) -> Result<Program, Errors> {
    build_with(src, Some(allowed_env))
}

fn build_with(src: &str, allowed_env: Option<&AllowedEnv>) -> Result<Program, Errors> {
    let mut doc: Document = loader::load(src)?;

    // the first block: the passport and the logging, before anything else
    let passport = doc.take(key::PASSPORT);
    let observability = doc.take(key::OBSERVABILITY);
    let trace = doc.take(key::TRACE);
    let allowed = network::allowed_hosts(passport.clone())?;
    let mut destinations = network::logging_destinations(observability.clone(), trace.clone())?;
    let log_url = Path::root().key(key::OBSERVABILITY).key(key::OUTPUT).key(key::URL);
    let mut refused = network::check(&allowed, destinations.iter().filter(|d| d.path == log_url).cloned());
    if let Some(env) = allowed_env {
        refused.extend(secrets::check(env, secrets::log_secrets(observability.clone())?));
    }
    if !refused.is_empty() {
        return Err(Errors(refused));
    }
    let verbose = Verbose::build(doc.take(key::VERBOSE), Redaction::from_passport(passport.clone())?)?;
    let trace_redaction = Redaction::from_passport(passport.clone())?;
    let logger = Logger::build(passport.clone(), observability)?;

    // 1. the rest, each block to its manager
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

    // 2. where the graph goes
    destinations.extend(models.destinations());
    destinations.extend(embeddings.destinations());
    destinations.extend(utilities.destinations());
    destinations.extend(resources.destinations());
    destinations.extend(external.destinations());
    destinations.extend(blackboards.destinations());
    let mut refused = network::check(&allowed, destinations);
    if let Some(env) = allowed_env {
        let mut used = secrets::trace_secrets(trace.clone())?;
        used.extend(models.secrets());
        used.extend(utilities.secrets());
        used.extend(resources.secrets());
        used.extend(external.secrets());
        refused.extend(secrets::check(env, used));
    }
    if !refused.is_empty() {
        return Err(report(&logger, refused));
    }

    // 3. the environment variables
    let mut models = models;
    let mut missing = models.connect().err().map(|e| e.0).unwrap_or_default();
    missing.extend(utilities.missing_env());
    missing.extend(resources.missing_env());
    let tracer = keep(Tracer::build(trace, passport, trace_redaction), &mut missing);
    if !missing.is_empty() {
        return Err(report(&logger, missing));
    }
    let tracer = tracer.flatten();

    // 4. the graph against the managers
    let mut broken = graph::interpret(&nodes, &edges, &models, &resources, &external);
    broken.extend(external.memory.check_prompts(&resources.prompts));
    if !broken.is_empty() {
        return Err(report(&logger, broken));
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
