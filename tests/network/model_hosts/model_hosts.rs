use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Every model host must be allowed.
#[test]
fn model_hosts() {
    assert!(refused(here(), "localhost", "models[5]"));
    assert!(refused(here(), "bedrock", "models[1]"));
    assert!(refused(here(), "azure", "models[4]"));
}

/// A `host` of a cloud provider replaces the host of its API: the gateway must be allowed, on its port, and the
/// host of the API is no longer needed.
#[test]
fn gateway_host() {
    assert_eq!(errors(here(), "gateway"), vec![]);
    assert!(refused(here(), "gateway_port", "models[0]"));
}
