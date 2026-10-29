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

/// A graph that is not valid is refused with 422 when the run is started: no run is created.
#[tokio::test]
async fn runs_refused() {
    let s = start(&config_path()).await;
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml", &[]), "message": "how long?" })).await;
    assert_eq!(status, 422);
    assert_eq!(body["errors"][0]["rule"], "unknown_ref");
    assert!(body.get("run").is_none());
}
