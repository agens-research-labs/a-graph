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
        "The only hosts a graph may reach, compared without regard to case. An entry `host` allows every port of the host; an entry `host:port` allows that port only. An entry is read as the host of a URL is read (`127.1` counts as `127.0.0.1`); an entry that is a URL or carries a path is refused. The `allowed_hosts` of the passport of a graph can only be a part of this list: an entry of the graph is accepted when this list has the same entry or the host alone. A graph that names another host, or a host without a port that this list allows only on a port, is refused.",
    ),
    Field::opt(RETENTION_SECONDS, Shape::Int).doc("How long a run is kept after it ends, so that it can still be watched and asked for; then it is forgotten. The default is 600, at least 1."),
    Field::opt(MAX_RUNS, Shape::Int).doc("The most runs going on at once, those of `POST /run` and of `POST /runs` together. A request beyond it is refused with 503 (`too_many_runs`), not queued. The default is 8, at least 1."),
    Field::opt(RUN_TIMEOUT_SECONDS, Shape::Int).doc("The longest a run may last. Then it is stopped and ends as failed (`run_timeout`). The default is 3600, at least 1."),
];
