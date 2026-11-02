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

/// With `session_max_bytes: 100` a session whose folder weighs 200 bytes is refused (422, `session_full`); another one
/// runs. A refused request, full or with a graph that is not valid, does not stamp the session.
#[tokio::test]
async fn session_full() {
    let data = std::path::Path::new("target/test-service/session_full");
    let _ = std::fs::remove_dir_all(data);
    std::fs::create_dir_all(data.join("s3")).unwrap();
    std::fs::write(data.join("s3/big.bin"), [0u8; 200]).unwrap();
    // a mark stamped long ago: a refused request must leave it so
    let long_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    let stamp_long_ago = |session: &str| {
        std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(false)
            .open(data.join(session).join(".a-graph-session"))
            .unwrap()
            .set_modified(long_ago)
            .unwrap()
    };
    let stamped = |session: &str| {
        std::fs::metadata(data.join(session).join(".a-graph-session"))
            .unwrap()
            .modified()
            .unwrap()
    };
    stamp_long_ago("s3");
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "s3" }))
        .await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(body["errors"][0]["rule"], "session_full");
    assert_eq!(body["errors"][0]["phase"], "execution");
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("more than session_max_bytes (100)"),
        "{body}"
    );
    assert!(
        body["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("ask the operator of the service to free it"),
        "{body}"
    );
    assert!(seen.lock().unwrap().is_empty(), "the graph did not run");
    assert_eq!(
        stamped("s3"),
        long_ago,
        "a request refused for a full session does not keep it alive"
    );
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "s4" }))
        .await;
    assert_eq!(status, 200, "{body}");
    // a session that weighs exactly the cap is not over it
    std::fs::create_dir_all(data.join("s6")).unwrap();
    std::fs::write(data.join("s6/exact.bin"), [0u8; 100]).unwrap();
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "s6" }))
        .await;
    assert_eq!(
        status, 200,
        "a session of exactly session_max_bytes is accepted: {body}"
    );
    // a graph that is not valid is refused, and does not keep the session alive either
    stamp_long_ago("s4");
    let (status, body) = s
        .run(json!({ "yml": "graph_spec: \"1.0\"\n", "message": "hi", "session": "s4" }))
        .await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(
        stamped("s4"),
        long_ago,
        "a request refused for its graph does not keep the session alive"
    );
}
