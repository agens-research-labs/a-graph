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

/// The events arrive while the graph is still running, not all at the end.
#[tokio::test]
async fn runs_live() {
    serve_at_slowly(19983, std::time::Duration::from_millis(600), |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19963).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    let id = body["run"].as_str().unwrap().to_string();
    let mut stream = reqwest::get(format!("http://127.0.0.1:{}/runs/{id}/events", s.port)).await.unwrap();
    // the first piece of the stream comes while the model is still thinking
    let first = stream.chunk().await.unwrap().unwrap();
    assert!(String::from_utf8_lossy(&first).contains("event: run.start"), "{first:?}");
    assert_eq!(s.get(&format!("/runs/{id}")).await.1["state"], "running", "the run is not over when the first event arrives");
    let mut rest = String::new();
    while let Some(chunk) = stream.chunk().await.unwrap() {
        rest.push_str(&String::from_utf8_lossy(&chunk));
    }
    assert!(rest.contains("event: result"), "{rest}");
}
