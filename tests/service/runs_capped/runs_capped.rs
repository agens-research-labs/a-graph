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

/// With `max_runs: 1` a request while a run goes on is refused with 503, before it is checked; after its end, it runs.
#[tokio::test]
async fn runs_capped() {
    serve_at_slowly(17401, std::time::Duration::from_secs(1), |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 17402).await;
    let body = json!({ "yml": read("graph.yml"), "message": "how long?" });
    let (status, started) = s.start_run(body.clone()).await;
    assert_eq!(status, 202, "{started}");
    let (status, refused) = s.run(body.clone()).await;
    assert_eq!(status, 503, "{refused}");
    assert_eq!(refused["errors"][0]["rule"], "too_many_runs");
    assert_eq!(refused["errors"][0]["path"], "<root>", "{refused}");
    // the refusal comes before the request is checked: a graph that is not valid is refused the same way
    let (status, refused) = s.start_run(json!({ "yml": "not a graph", "message": "how long?" })).await;
    assert_eq!(status, 503, "{refused}");
    assert_eq!(refused["errors"][0]["rule"], "too_many_runs");
    let id = started["run"].as_str().unwrap();
    assert_eq!(s.finished(id).await["state"], "done");
    let (status, done) = s.run(body).await;
    assert_eq!(status, 200, "{done}");
}
