//! a-graph: loading and validation of the yml, the managers built from it, and the logging of what happens.

pub mod blackboards;
mod build;
pub mod embeddings;
pub mod error;
pub mod external;
pub mod gate;
pub mod graph;
pub mod loader;
pub mod logging;
pub mod models;
pub mod resources;
pub mod run;
pub mod service;
pub mod utilities;
pub mod schema;
mod net;
pub mod network;

pub use error::{Error, Errors, Path, Phase, Rule, Warning};
