/// `allowed_ambient` lists only the providers that have `credentials: ambient`: another word keeps
/// the service from starting.
#[test]
fn ambient_config() {
    let base = "port: 1\ndata_dir: d\nallowed_hosts: [h]\nallowed_env: {}\n";
    let errs = a_graph::service::ServiceConfig::read(&format!("{base}allowed_ambient: [bedrock, ollama]")).unwrap_err();
    let got: Vec<(&str, String)> = errs.0.iter().map(|e| (e.rule.code(), e.path.to_string())).collect();
    assert_eq!(got, [("bad_value", "allowed_ambient[1]".to_string())]);

    let config =
        a_graph::service::ServiceConfig::read(&format!("{base}allowed_ambient: [bedrock, vertex_ai, azure_foundry]"))
            .unwrap();
    assert_eq!(config.allowed_ambient, ["bedrock", "vertex_ai", "azure_foundry"]);
}
