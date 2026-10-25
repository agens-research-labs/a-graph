use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// A graph that would start programs is refused with 422: stdio tools, MCP servers and utilities, and the scripts of a skill.
#[tokio::test]
async fn no_programs() {
    let (_, seen) = serve_at(19995, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19975).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml"), "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    let at: Vec<&str> = body["errors"].as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap()).collect();
    for wanted in ["utilities[0].transport", "resources.tools[0].transport", "resources.mcp_servers[0].transport", "resources.skills[0]"] {
        assert!(at.contains(&wanted), "no error at `{wanted}` in {at:?}");
    }
    assert!(body["errors"].as_array().unwrap().iter().all(|e| e["rule"] == "not_supported"));
    assert!(seen.lock().unwrap().is_empty(), "nothing was run");
}
