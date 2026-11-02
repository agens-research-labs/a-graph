use crate::common::own_yml;
use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Secrets keep the name of the variable not a value.
#[test]
fn secrets_keep_names() {
    let r = build_text(&own_yml(here())).unwrap();
    let McpTransport::Http { headers, .. } = &r.mcp.get("remote").unwrap().transport else {
        panic!()
    };
    assert_eq!(headers["Authorization"], "MCP_TOKEN");
}
