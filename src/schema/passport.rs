//! `passport`: the ADL 0.3.0 identity of the agent.

use super::key::*;
use super::{Field, Shape};

pub(super) const PASSPORT: Shape = Shape::Map(&[Field::req(ADL, ADL_BODY)
    .doc("The passport, in the ADL format (https://www.adl-spec.org/). The key names are those of ADL.")]);

const ADL_BODY: Shape = Shape::Map(&[
    Field::req(ADL_SPEC, Shape::Text).doc("The version of ADL the passport is written in: `\"0.3.0\"`."),
    Field::req(NAME, Shape::Text).doc("The name of the agent. The trace uses it as the name of the service."),
    Field::req(DESCRIPTION, Shape::Text).doc("What the agent does."),
    Field::req(VERSION, Shape::Text).doc("The version of the agent, as a semantic version of three numbers (for example `\"1.0.0\"`)."),
    Field::req(DATA_CLASSIFICATION, DATA_CLASSIFICATION_BODY)
        .doc("How sensitive the data of the graph are. It decides whether the contents (prompts, answers, results) are written in the log, the trace and the display."),
    Field::opt(LIFECYCLE, LIFECYCLE_BODY).doc("The stage of life of the agent. The program does not act on it."),
    Field::opt(PERMISSIONS, PERMISSIONS_BODY).doc("What the graph is allowed to do. Without it, no host is allowed."),
]);

const DATA_CLASSIFICATION_BODY: Shape = Shape::Map(&[
    Field::req(
        SENSITIVITY,
        Shape::Words(&["public", "internal", "confidential", "restricted"]),
    )
    .doc("The level of sensitivity of the data. The contents are written only with `public` and no `categories`."),
    Field::opt(
        CATEGORIES,
        Shape::List(&Shape::Words(&[
            "pii",
            "phi",
            "financial",
            "credentials",
            "intellectual_property",
            "regulatory",
        ])),
    )
    .doc("The kinds of regulated data the graph handles. Any category hides the contents."),
]);

const LIFECYCLE_BODY: Shape = Shape::Map(&[
    Field::req(STATUS, Shape::Words(&["draft", "active", "deprecated", "retired"])).doc("The stage the agent is in."),
    Field::opt(SUNSET_DATE, Shape::Text)
        .doc("When the agent is retired, in RFC 3339 with a time zone (for example `2027-01-01T00:00:00Z`)."),
]);

const PERMISSIONS_BODY: Shape = Shape::Map(&[Field::opt(
    NETWORK,
    Shape::Map(&[Field::req(ALLOWED_HOSTS, Shape::List(&Shape::Text)).doc(
        "The hosts the graph may reach: host names, not URLs (`api.anthropic.com`, not `https://api.anthropic.com/v1`), compared without regard to case. An entry `host` allows every port of the host; an entry `host:port` (`localhost:11434`, `[::1]:8080`) allows that port only. The port of a URL is the one written in it, or 80 for `http` and 443 for `https`; the host of a provider is reached on 443. An entry is read as the host of a URL is read: `127.1` counts as `127.0.0.1`, `[0:0:0:0:0:0:0:1]` as `[::1]`. An entry that is a URL or carries a path is refused. Every destination of the file is checked against this list before anything runs.",
    )]),
)
.doc("The network the graph may reach. Without it, no host is allowed.")]);
