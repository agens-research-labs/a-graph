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

/// A graph with `verbose: off` streams no event of the steps: only the result.
#[tokio::test]
async fn runs_level_off() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (_, body) = s.start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "how long?" })).await;
    let events = s.events(body["run"].as_str().unwrap()).await;
    assert_eq!(events.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["result"]);
}
