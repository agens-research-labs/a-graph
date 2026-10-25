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

/// A graph that allows a host the service does not allow is refused with 422.
#[tokio::test]
async fn host_refused() {
    let s = start(&config_path(), 19977).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml"), "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    let e = &body["errors"][0];
    assert_eq!((e["rule"].as_str(), e["path"].as_str()), (Some("network_not_allowed"), Some("passport.adl.permissions.network.allowed_hosts[1]")));
}
