//! The file of the service, `a-graph --serve service.yml`: not a graph, but the choices of the
//! owner of the service. It has no keys of models or services: those are variables of the
//! container, which the graphs name as always.

use super::key::*;
use super::{Field, Shape};

pub const SERVICE: &[Field] = &[
    Field::req(PORT, Shape::Int).doc("The port the service listens on, on all interfaces."),
    Field::req(DATA_DIR, Shape::Text).doc(
        "The folder of the sessions. Each session has its own folder, `<data_dir>/<session>/`, and every path of a graph sent to the service is moved inside it.",
    ),
    Field::req(ALLOWED_HOSTS, Shape::List(&Shape::Text)).doc(
        "The only hosts a graph may reach, compared without regard to case. The `allowed_hosts` of the passport of a graph can only be a part of this list: a graph that names another host is refused.",
    ),
    Field::opt(RETENTION_SECONDS, Shape::Int).doc("How long a run is kept after it ends, so that it can still be watched and asked for; then it is forgotten. The default is 600, at least 1."),
];
