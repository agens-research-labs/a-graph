use crate::common::logging_logger::*;

/// An event field cannot overwrite the fixed keys.
#[test]
fn fixed_keys() {
    let (log, buf) = logger(LogLevel::Debug, true);
    log.log(&Event::new(LogLevel::Info, "x").field("code", "forged"));
    assert_eq!(buf.lines()[0]["code"], "x");
}
