//! The tool loop. Every test is a folder with its own graph; the model is a mock
//! server (on a free port) that asks for the tool and then answers `R(<what the tool
//! gave back>)`, so the answer shows what reached the model. The tools are the small scripts of
//! `files/tools/` of the folder of each test and, for http, a second mock server.

pub use std::sync::{Arc, Mutex};

pub use serde_json::{Value, json};

pub use a_graph::run::{Content, Program, RunOutput, build};

pub use super::run::{yml_in, yml_with};

pub use super::http::{Req, Resp, listen};

pub async fn program(
    dir: &std::path::Path,
    name: &str,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    program_with(dir, name, &[], handler).await
}

/// Like `program`, with the ports of the other mock servers the test started (the http tools).
pub async fn program_with(
    dir: &std::path::Path,
    name: &str,
    ports: &[(&str, u16)],
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    let (port, seen) = listen(handler).await;
    let yml = yml_with(dir, name, &[ports, &[("model", port)]].concat());
    (Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

pub use super::toolmock::*;
