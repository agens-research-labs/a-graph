//! `observability`, `trace`, `verbose`: what the agent reports about itself.

use super::key::*;
use super::{Field, SECRET, Shape};

const HEADERS_OF_SECRETS: Shape = Shape::MapOf(&SECRET);

pub(super) const OBSERVABILITY: Shape = Shape::Map(&[
    Field::req(LEVEL, Shape::Words(&["debug", "info", "warn", "error"]))
        .doc("The lowest level written: `info` writes the run, the nodes and the groups, `debug` also the calls, `warn` and `error` only the problems."),
    Field::opt(
        OUTPUT,
        Shape::Either(&[
            Shape::Words(&["stdout", "stderr"]),
            Shape::Map(&[Field::req(FILE, Shape::Text).doc("The file the log is added to, created if it does not exist.")]),
            Shape::Map(&[
                Field::req(URL, Shape::Text).doc("The HTTP endpoint the log is sent to, as POSTs of `application/x-ndjson` with at most 100 events each."),
                Field::opt(HEADERS, HEADERS_OF_SECRETS).doc("The headers of the POSTs, each read from an environment variable."),
            ]),
        ]),
    )
    .doc("Where the log goes: `stderr` (the default), with the errors, so that the standard output holds only the result; `stdout`, mixed with the result; a file; or an HTTP endpoint. A log that cannot be written never stops the run."),
]);

pub(super) const TRACE: Shape = Shape::Map(&[
    Field::req(ENDPOINT, Shape::Text).doc(
        "The OpenTelemetry collector, as the exact URL of the POST (for example `http://host:4318/v1/traces`). At the end of the run, also when it fails, the whole timeline is sent in one POST, in OTLP over HTTP with JSON. A POST that fails is tried again up to 3 times; what cannot be delivered is dropped, with a notice on the standard error.",
    ),
    Field::opt(HEADERS, HEADERS_OF_SECRETS).doc("The headers of the POST, each read from an environment variable."),
    Field::opt(CAPTURE_CONTENT, Shape::Bool).doc("Whether trace attributes include prompts, answers, reasoning and error details. The default is false. Enabling this never overrides passport redaction; logs and the live display retain their own behavior."),
    Field::opt(MAX_CONTENT_BYTES, Shape::Int).doc("The most UTF-8 bytes of each trace content attribute, at least 1; the default is 4096. Truncation preserves character boundaries and adds a_graph.content_truncated=true."),
]);

pub(super) const VERBOSE: Shape = Shape::Map(&[Field::req(LEVEL, Shape::Words(&["off", "steps", "detail"])).doc(
    "What is shown: `off` nothing; `steps` the run, the nodes and the groups as they start and end, and the errors and warnings; `detail` also the calls, the readings and the contents, as the passport allows.",
)]);
