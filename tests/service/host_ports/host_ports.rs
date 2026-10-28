use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn config_path(name: &str) -> String {
    dir().join(name).to_string_lossy().to_string()
}

/// A graph entry `host:port` is allowed by the same entry or by `host` alone; a graph entry `host`
/// is refused by a service that allows only `host:port`, because it would allow every port.
#[tokio::test]
async fn host_ports() {
    let (_, _) = serve_at(17463, |_| Resp::ok(answer("ok"))).await;
    let any_port = start(&config_path("service_host.yml"), 17461).await;
    let one_port = start(&config_path("service_port.yml"), 17462).await;

    let (status, body) = any_port.run(json!({ "yml": read("graph_port.yml"), "message": "x" })).await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = one_port.run(json!({ "yml": read("graph_port.yml"), "message": "x" })).await;
    assert_eq!(status, 200, "{body}");

    let (status, body) = one_port.run(json!({ "yml": read("graph_host.yml"), "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    let e = &body["errors"][0];
    assert_eq!((e["rule"].as_str(), e["path"].as_str()), (Some("network_not_allowed"), Some("passport.adl.permissions.network.allowed_hosts[0]")));
}

/// A port of the service file that is not a number from 1 to 65535 is a bad value.
#[test]
fn host_ports_bad() {
    let errs = a_graph::service::ServiceConfig::read("port: 1\ndata_dir: d\nallowed_hosts: [\"h:0\", h]").unwrap_err();
    let got: Vec<(&str, String)> = errs.0.iter().map(|e| (e.rule.code(), e.path.to_string())).collect();
    assert_eq!(got, vec![("bad_value", "allowed_hosts[0]".to_string())]);
}
