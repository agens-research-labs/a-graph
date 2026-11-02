use crate::common::own_yml;
use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A tool has a max iter of at least one.
#[test]
fn tool_max_iter() {
    assert!(has(
        here(),
        "tool_iter_zero",
        "bad_value",
        "resources.tools[0].max_iter"
    ));
    assert_eq!(
        build_text(&own_yml(here()))
            .unwrap()
            .tools
            .get("get_weather")
            .unwrap()
            .max_iter,
        5
    );
    assert!(has(here(), "tool_time_zero", "bad_value", "resources.tools[0].timeout"));
    assert!(has(
        here(),
        "mcp_iter_zero",
        "bad_value",
        "resources.mcp_servers[0].max_iter"
    ));
    assert!(has(
        here(),
        "mcp_time_zero",
        "bad_value",
        "resources.mcp_servers[0].timeout"
    ));
    assert_eq!(
        build_text(&own_yml(here())).unwrap().mcp.get("files").unwrap().max_iter,
        5
    );
    assert_eq!(
        build_text(&own_yml(here()))
            .unwrap()
            .tools
            .get("get_weather")
            .unwrap()
            .timeout,
        30
    );
}
