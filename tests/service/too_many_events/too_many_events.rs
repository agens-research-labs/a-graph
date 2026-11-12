use crate::common::service::*;

fn config_path() -> String {
    std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join("service.yml")
        .to_string_lossy()
        .to_string()
}

/// `x: [0, ...]` with `n` values has `n + 9` events: the stream, the document, the map and the list (start and end
/// each), the key `x` and the `n` values.
fn values(n: usize) -> String {
    format!("x: [{}]", vec!["0"; n].join(","))
}

/// A yml of more than 100000 YAML events is refused (`not_supported`) before it is read whole; one of 100000 is read,
/// and refused only for what it says.
#[tokio::test]
async fn too_many_events() {
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": values(99_992), "message": "m" })).await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(body["errors"].as_array().unwrap().len(), 1, "{body}");
    assert_eq!(body["errors"][0]["rule"], "not_supported", "{body}");
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("100000 YAML events"),
        "{body}"
    );
    let (status, body) = s.run(json!({ "yml": values(99_991), "message": "m" })).await;
    assert_eq!(status, 422, "{body}");
    assert!(
        body["errors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["rule"] != "not_supported"),
        "{body}"
    );
}
