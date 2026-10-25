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

/// The events of a run in a graph with `verbose: detail`: the steps, the calls, and `result` as the last.
#[tokio::test]
async fn runs_events() {
    serve_at(19981, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19961).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    let id = body["run"].as_str().unwrap().to_string();
    let events = s.events(&id).await;
    let names: Vec<&str> = events.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names.first(), Some(&"run.start"));
    assert_eq!(&names[names.len() - 2..], ["run.end", "result"], "{names:?}");
    for wanted in ["node.start", "model.call", "node.end"] {
        assert!(names.contains(&wanted), "no `{wanted}` in {names:?}");
    }
    let call = events.iter().find(|(n, _)| n == "model.call").unwrap();
    assert_eq!((call.1["node"].as_str(), call.1["model"].as_str(), call.1["input_tokens"].as_u64()), (Some("a"), Some("m"), Some(1)));
    assert_eq!(call.1["prompt"], "Q: how long?", "the data is public: the text is there");
    assert_eq!(events.last().unwrap().1["items"][0]["node"], "b");
}
