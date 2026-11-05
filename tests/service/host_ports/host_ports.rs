use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path(name: &str) -> String {
    dir().join(name).to_string_lossy().to_string()
}

/// A graph entry `host:port` is allowed by the same entry or by `host` alone; a graph entry `host`
/// is refused by a service that allows only `host:port`, because it would allow every port.
#[tokio::test]
async fn host_ports() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let any_port = start(&config_path("service_host.yml")).await;
    let one_port = start_with(&config_path("service_port.yml"), &[("model", model)]).await;

    let (status, body) = any_port
        .run(json!({ "yml": read("graph_port.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = one_port
        .run(json!({ "yml": read("graph_port.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 200, "{body}");

    let (status, body) = one_port
        .run(json!({ "yml": read("graph_host.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 422, "{body}");
    let e = &body["errors"][0];
    assert_eq!(
        (e["rule"].as_str(), e["path"].as_str()),
        (
            Some("network_not_allowed"),
            Some("passport.adl.permissions.network.allowed_hosts[0]")
        )
    );
}

/// A port of the service file that is not a number from 1 to 65535, or an entry that is a URL, is a bad value.
#[test]
fn host_ports_bad() {
    let errs = a_graph::service::ServiceConfig::read(
        "port: 1\ndata_dir: d\nallowed_hosts: [\"h:0\", h, \"https://h\"]\nallowed_env: {}",
    )
    .unwrap_err();
    let got: Vec<(&str, String)> = errs.0.iter().map(|e| (e.rule.code(), e.path.to_string())).collect();
    assert_eq!(
        got,
        vec![
            ("bad_value", "allowed_hosts[0]".to_string()),
            ("bad_value", "allowed_hosts[2]".to_string())
        ]
    );
    let e = &errs.0[1];
    assert!(e.message.contains("is a URL"), "{}", e.message);
}

/// The entries of the service file are kept as the host of a URL is read, so a graph entry written
/// another way for the same host still matches.
#[test]
fn host_ports_normalized() {
    let config = a_graph::service::ServiceConfig::read(
        "port: 1\ndata_dir: d\nallowed_hosts: [\"127.1\", \"[0:0:0:0:0:0:0:1]:11434\", Api.Example.COM]\nallowed_env: {}",
    )
    .unwrap();
    assert_eq!(
        config.allowed_hosts,
        vec!["127.0.0.1", "[::1]:11434", "api.example.com"]
    );
}
