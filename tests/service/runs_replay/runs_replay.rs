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

/// Someone who connects after the end gets the same events, and the result.
#[tokio::test]
async fn runs_replay() {
    serve_at(19982, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19962).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    let id = body["run"].as_str().unwrap().to_string();
    let live = s.events(&id).await;
    s.finished(&id).await;
    assert_eq!(s.events(&id).await, live);
    assert_eq!(s.events(&id).await.last().unwrap().0, "result");
}
