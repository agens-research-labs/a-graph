use crate::common::logging_logger::*;

/// Content is redacted unless allowed.
#[test]
fn redaction() {
    let ev = Event::new(LogLevel::Info, "run.model.call").node("a").field("tokens", 42).content("response", "secret text");

    let (log, buf) = logger(LogLevel::Debug, false);
    log.log(&ev);
    let l = &buf.lines()[0];
    assert_eq!(l["tokens"], 42);
    assert_eq!(l["node"], "a");
    assert_eq!(l["content_redacted"], true);
    assert!(l.get("response").is_none());

    let (log, buf) = logger(LogLevel::Debug, true);
    log.log(&ev);
    assert_eq!(buf.lines()[0]["response"], "secret text");
}
