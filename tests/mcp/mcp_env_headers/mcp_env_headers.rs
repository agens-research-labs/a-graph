use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Mcp env is for stdio and headers for http.
#[test]
fn mcp_env_headers() {
    assert!(has(here(), "mcp_env_headers", "unexpected_field", "resources.mcp_servers[1].env"));
    assert!(has(here(), "mcp_env_headers", "unexpected_field", "resources.mcp_servers[0].headers"));
}
