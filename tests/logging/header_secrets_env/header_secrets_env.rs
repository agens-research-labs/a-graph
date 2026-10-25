use crate::common::logging_delivery::*;
use crate::common::{yml_in};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Build reads header secrets from the environment.
#[tokio::test]
async fn header_secrets_env() {
    let (passport, observability) = blocks(&yml_in(here(), "header_unset"));
    let err = Logger::build(passport, observability).err().unwrap();
    assert_eq!(err.0[0].rule.code(), "env_missing");
    assert_eq!(err.0[0].path.to_string(), "observability.output.headers.Authorization");

    let (passport, observability) = blocks(&yml_in(here(), "url"));
    let logger = Logger::build(passport, observability).unwrap();
    logger.close().await;
}
