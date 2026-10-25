//! The service: `a-graph --serve service.yml` answers `POST /run` with the yml of a graph, a
//! message and, if wanted, a session; and the other routes of `api`. The yml comes from outside, so it is not
//! trusted: `rules` refuses what could start a program, read or write a file outside the folder of
//! the session or reach a host the service does not allow, before anything is built.

mod api;
mod config;
mod rules;
mod runs;
mod server;

pub use api::{Endpoint, openapi};
pub use config::ServiceConfig;
pub use rules::prepare;
pub use server::serve;
