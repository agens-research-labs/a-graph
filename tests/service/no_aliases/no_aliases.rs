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

/// A graph with an alias is refused with 422, and nothing is run.
#[tokio::test]
async fn no_aliases() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 422, "{body}");
    let errors = body["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1, "{body}");
    assert_eq!(errors[0]["rule"], "not_supported");
    assert!(errors[0]["message"].as_str().unwrap().contains("aliases"), "{body}");
    assert!(seen.lock().unwrap().is_empty(), "nothing was run");
}

/// An alias that would copy 1 MiB 200 times is refused before it is copied: the answer comes at once.
#[tokio::test]
async fn no_aliases_bomb() {
    let s = start(&config_path()).await;
    let mut yml = format!("big: &x \"{}\"\nlist:\n", "a".repeat(1 << 20));
    yml.push_str(&"  - *x\n".repeat(200));
    let t = std::time::Instant::now();
    let (status, body) = s.run(json!({ "yml": yml, "message": "x" })).await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(body["errors"][0]["rule"], "not_supported", "{body}");
    assert!(t.elapsed() < std::time::Duration::from_secs(5), "{:?}", t.elapsed());
}

/// `*` and `&` inside a text are not aliases: the graph runs.
#[tokio::test]
async fn no_aliases_in_texts() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("plain.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 200, "{body}");
    let first: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(first["messages"][1]["content"], "a * b & c, *x &y: x");
}
