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

/// A graph with `verbose: steps` streams the nodes but not the calls.
#[tokio::test]
async fn runs_level_steps() {
    serve_at(19985, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19965).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    let events = s.events(body["run"].as_str().unwrap()).await;
    let names: Vec<&str> = events.iter().map(|(n, _)| n.as_str()).collect();
    assert!(names.contains(&"node.start") && names.contains(&"node.end") && !names.contains(&"model.call"), "{names:?}");
}
