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

/// The (rule, path) of every error of the answer.
fn errors(body: &Value) -> Vec<(String, String)> {
    body["errors"]
        .as_array()
        .unwrap_or_else(|| panic!("no errors in {body}"))
        .iter()
        .map(|e| {
            (
                e["rule"].as_str().unwrap().to_string(),
                e["path"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// A graph that names a variable the service does not list is refused, and the variable is not
/// read: the answer is the same whether it is set or not, never `env_missing`.
#[tokio::test]
async fn env_not_listed() {
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[]), "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(
        errors(&body),
        [(
            "env_not_allowed".to_string(),
            "resources.tools[0].headers.X-Key".to_string()
        )]
    );
}
