use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// `POST /runs` answers 202 at once with an id; the state is `running` and then `done` with the same result as `/run`.
#[tokio::test]
async fn runs_result() {
    let (model, _) = listen_slowly(std::time::Duration::from_millis(300), |_| Resp::ok(answer("the lease lasts 12 months"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" })).await;
    assert_eq!(status, 202, "{body}");
    let id = body["run"].as_str().unwrap().to_string();
    assert!(id.starts_with("r-"));
    assert_eq!(s.get(&format!("/runs/{id}")).await, (200, json!({ "state": "running" })));
    let done = s.finished(&id).await;
    assert_eq!(done["state"], "done");
    assert_eq!(done["result"], json!({ "items": [{ "node": "b", "message": "the lease lasts 12 months", "media": [], "reasoning": [] }] }));
}
