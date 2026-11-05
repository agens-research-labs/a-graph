/// The (rule, path) of every error of the service file.
fn errors(text: &str) -> Vec<(String, String)> {
    a_graph::service::ServiceConfig::read(text)
        .unwrap_err()
        .0
        .iter()
        .map(|e| (e.rule.code().to_string(), e.path.to_string()))
        .collect()
}

/// A variable of `allowed_env` without hosts, or with a host that `allowed_hosts` does not allow,
/// keeps the service from starting: no graph could send it there. An entry is read as an entry of
/// `allowed_hosts` is, and the ports follow the same rule.
#[test]
fn env_config() {
    let base = "port: 1\ndata_dir: d\nallowed_hosts: [api.example.com, \"localhost:11434\"]\nallowed_ambient: []\n";
    assert_eq!(
        errors(&format!(
            "{base}allowed_env: {{ A: [], B: [other.example.com], C: [localhost], D: [\"https://api.example.com\"] }}"
        )),
        [
            ("bad_value".to_string(), "allowed_env.A".to_string()),
            ("network_not_allowed".to_string(), "allowed_env.B[0]".to_string()),
            ("network_not_allowed".to_string(), "allowed_env.C[0]".to_string()),
            ("bad_value".to_string(), "allowed_env.D[0]".to_string()),
        ]
    );

    let config = a_graph::service::ServiceConfig::read(&format!(
        "{base}allowed_env: {{ K: [API.example.com, \"localhost:11434\"] }}"
    ))
    .unwrap();
    let hosts = &config.allowed_env.0["K"];
    assert!(
        hosts.contains("api.example.com") && hosts.contains("localhost:11434"),
        "{hosts:?}"
    );

    let config = a_graph::service::ServiceConfig::read(&format!("{base}allowed_env: {{}}")).unwrap();
    assert!(config.allowed_env.0.is_empty());
}
