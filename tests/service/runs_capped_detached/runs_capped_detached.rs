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

/// A caller of `POST /run` that hangs up does not free the permit: the run goes on and holds it until its end.
#[tokio::test]
async fn runs_capped_detached() {
    let (_, seen) = serve_at_slowly(17407, std::time::Duration::from_secs(2), |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 17408).await;
    let body = json!({ "yml": read("graph.yml"), "message": "how long?" });
    let first = tokio::spawn({
        let body = body.clone();
        async move { reqwest::Client::new().post("http://127.0.0.1:17408/run").json(&body).send().await }
    });
    // the run is going on once the model is called: then the caller hangs up
    for _ in 0..100 {
        if !seen.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(seen.lock().unwrap().len(), 1, "the model was not called");
    first.abort();
    let _ = first.await;
    let (status, refused) = s.run(body.clone()).await;
    assert_eq!(status, 503, "{refused}");
    assert_eq!(refused["errors"][0]["rule"], "too_many_runs");
    // after the end of the first run, the permit is back
    for _ in 0..50 {
        let (status, _) = s.run(body.clone()).await;
        if status == 200 {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("the permit never came back");
}
