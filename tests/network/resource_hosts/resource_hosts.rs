use crate::common::network::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Resources and external destinations must be allowed.
#[test]
fn resource_hosts() {
    assert!(refused(here(), "mcp", "resources.mcp_servers[1].url"));
    assert!(refused(here(), "qdrant", "external.vectordb[0].url"));
}
