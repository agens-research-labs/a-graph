use crate::common::logging_logger::*;

/// A warning becomes a warn event.
#[test]
fn warning_event() {
    let (log, buf) = logger(LogLevel::Warn, false);
    log.log(&Event::from(&Warning::new(Phase::Interpretation, Path::root().key("resources").key("skills").idx(0).key("path"), "unknown_field", "`argument-hint` is not defined")));
    let l = &buf.lines()[0];
    assert_eq!((l["level"].as_str(), l["code"].as_str(), l["phase"].as_str()), (Some("warn"), Some("unknown_field"), Some("interpretation")));
    assert_eq!(l["path"], "resources.skills[0].path");
}
