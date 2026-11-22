use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// The judge answers a verdict without `validation`, every other node the text it was asked.
fn judge(req: &crate::common::http::Req) -> Resp {
    let body: Value = serde_json::from_str(&req.body).unwrap();
    if body["format"]["properties"]["validation"].is_object() {
        let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
        return Resp::ok(answer(&verdict.to_string()));
    }
    Resp::ok(answer("ok"))
}

/// The caller of a run that fails gets the message of the error without what made it: in the answer
/// of `POST /run`, in the state of `GET /runs/{id}` and in the stream of its events.
#[tokio::test]
async fn error_kept_out() {
    let (model, _) = listen(judge).await;
    let s = start(&config_path()).await;
    let request = json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi" });

    let (status, body) = s.run(request.clone()).await;
    assert_eq!(status, 502, "{body}");
    assert_eq!(body["errors"][0]["rule"], "schema_mismatch", "{body}");
    assert!(!body.to_string().contains("diabetes"), "{body}");

    let (_, started) = s.start_run(request).await;
    let id = started["run"].as_str().unwrap().to_string();
    let events = s.events(&id).await;
    let shown: Vec<String> = events.iter().map(|(n, d)| format!("{n} {d}")).collect();
    assert!(!shown.join("\n").contains("diabetes"), "{shown:?}");
    assert_eq!(events.last().unwrap().0, "error", "{shown:?}");
    let state = s.finished(&id).await;
    assert_eq!(state["errors"][0]["rule"], "schema_mismatch", "{state}");
    assert!(!state.to_string().contains("diabetes"), "{state}");
}
