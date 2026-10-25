use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Headers are for http tools and their variable must be set.
#[test]
fn tool_headers() {
    assert!(has(here(), "stdio", "unexpected_field", "resources.tools[0].headers"));
    assert!(has(here(), "http_env_missing", "env_missing", "resources.tools[0].headers.X-Key"));
    // the value is never kept: only the name of the variable
    unsafe { std::env::set_var("ANTHROPIC_API_KEY", "a") };
    let r = build(here(), "http_ok").unwrap();
    assert!(matches!(&r.tools.get("get_weather").unwrap().transport, ToolTransport::Http { headers, .. } if headers["X-Key"] == "ANTHROPIC_API_KEY"));
}
