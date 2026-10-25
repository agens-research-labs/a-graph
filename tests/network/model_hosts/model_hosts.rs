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
