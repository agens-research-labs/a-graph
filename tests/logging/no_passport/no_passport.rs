use crate::common::logging_logger::*;

/// A missing passport is an error.
#[test]
fn no_passport() {
    assert!(Redaction::from_passport(None).is_err());
}
