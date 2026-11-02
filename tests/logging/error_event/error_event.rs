use crate::common::logging_logger::*;

/// An error becomes an event.
#[test]
fn error_event() {
    let (log, buf) = logger(LogLevel::Error, false);
    log.log_error(&Error::new(
        Phase::Validation,
        Path::root().key("graph").key("nodes").idx(2),
        Rule::UnknownRef,
        "model `x` is not declared",
    ));
    let l = &buf.lines()[0];
    assert_eq!(l["level"], "error");
    assert_eq!(l["code"], "unknown_ref");
    assert_eq!(l["phase"], "validation");
    assert_eq!(l["path"], "graph.nodes[2]");
}
