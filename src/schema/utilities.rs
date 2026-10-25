//! `utilities`: programs and services that the program (never the model) calls on the data of a node.

use super::key::*;
use super::{Field, Kind, Shape};

const UTILITY: Shape = Shape::Map(&[
    Field::req(ID, Shape::Id(Kind::Utility)).doc("The id of the utility, cited by the nodes where they want their data changed."),
    Field::req(TRANSPORT, Shape::Words(&["stdio", "http"]))
        .doc("`stdio`: the program `command` is started for each call. `http`: a service already running, which the program does not start or stop."),
    Field::opt(COMMAND, Shape::Text).doc("The program to start, with `transport: stdio` only (where it is mandatory)."),
    Field::opt(ARGS, Shape::List(&Shape::Text)).doc("The arguments of `command`, with `transport: stdio` only."),
    Field::opt(URL, Shape::Text).doc("The URL the calls are POSTed to, with `transport: http` only (where it is mandatory). Its host must be in `allowed_hosts`."),
    Field::opt(HEADERS, Shape::MapOf(&super::SECRET)).doc("The headers of the POSTs, with `transport: http` only, each read from an environment variable when the file is checked."),
    Field::req(TIMEOUT, Shape::Int).doc("The seconds a call may take, at least 1. After that the program is stopped or the request abandoned, and the run fails."),
]);

pub(super) const UTILITIES: Shape = Shape::List(&UTILITY);
