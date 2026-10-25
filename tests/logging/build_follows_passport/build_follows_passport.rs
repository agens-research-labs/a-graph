use crate::common::logging_logger::*;
use crate::common::{yml_in, own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Build follows the passport and the output.
#[test]
fn build_follows_passport() {
    // internal + pii: no content
    let (passport, _) = blocks(&own_yml(here()));
    assert!(!Redaction::from_passport(passport).unwrap().allow_content);
    let (passport, _) = blocks(&yml_in(here(), "public"));
    assert!(Redaction::from_passport(passport).unwrap().allow_content);

    // a network output without a tokio runtime
    let (passport, observability) = blocks(&yml_in(here(), "url"));
    let err = Logger::build(passport, observability).err().unwrap();
    assert_eq!(err.0[0].rule.code(), "not_supported");

    let (passport, observability) = blocks(&yml_in(here(), "missing_dir"));
    let err = Logger::build(passport, observability).err().unwrap();
    assert_eq!(err.0[0].rule.code(), "io");
}
