//! The tool loop. Every test is a folder with its own graph; the model is a mock
//! server (on the port the file names) that asks for the tool and then answers `R(<what the tool
//! gave back>)`, so the answer shows what reached the model. The tools are the small scripts of
//! `files/tools/` of the folder of each test and, for http, a second mock server.

pub use std::sync::{Arc, Mutex};

pub use serde_json::{Value, json};

pub use a_graph::run::{Content, Program, RunOutput, build};

pub use super::run::{yml_in};

pub use super::http::{Req, Resp, serve_at};

pub async fn program(dir: &std::path::Path, name: &str, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    let yml = yml_in(dir, name);
    let (_, seen) = serve_at(port_of(&yml), handler).await;
    (Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

pub use super::toolmock::*;
