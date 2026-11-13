//! a-graph: loading and validation of the yml, the managers built from it, and the logging of what happens.

pub mod blackboards;
mod build;
pub mod embeddings;
pub mod error;
pub mod external;
mod files;
pub mod gate;
pub mod graph;
pub mod loader;
pub mod logging;
pub mod models;
mod net;
pub mod network;
pub mod resources;
pub mod run;
pub mod schema;
pub mod secrets;
pub mod service;
pub mod utilities;

pub use error::{Error, Errors, Path, Phase, Rule, Warning};
