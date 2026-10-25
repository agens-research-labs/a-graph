use crate::common::logging_logger::*;

/// Drops events below the level.
#[test]
fn level_filter() {
    let (log, buf) = logger(LogLevel::Warn, true);
    log.log(&Event::new(LogLevel::Info, "run.node.started"));
    log.log(&Event::new(LogLevel::Warn, "run.retry"));
    let lines = buf.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["code"], "run.retry");
    assert_eq!(lines[0]["level"], "warn");
}
