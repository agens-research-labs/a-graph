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

/// Two sessions keep their own memory in their own folder; without a session the folder is `default`.
#[tokio::test]
async fn sessions_apart() {
    // the sessions keep their files on disk: start from nothing
    let _ = std::fs::remove_dir_all("target/test-service/sessions_apart");
    let (_, seen) = serve_at(19998, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19978).await;
    let ask = |session: Option<&str>, message: &str| {
        let mut body = json!({ "yml": read("graph.yml"), "message": message });
        if let Some(name) = session {
            body["session"] = json!(name);
        }
        s.run(body)
    };
    assert_eq!(ask(Some("ann"), "first of ann").await.0, 200);
    assert_eq!(ask(Some("bob"), "first of bob").await.0, 200);
    assert_eq!(ask(Some("ann"), "second of ann").await.0, 200);
    assert_eq!(ask(None, "nobody").await.0, 200);

    // the request that goes to node a of ann's second turn has ann's first exchange as history, and nothing of bob
    let asked: Vec<Value> = seen.lock().unwrap().iter().map(|r| serde_json::from_str::<Value>(&r.body).unwrap()).collect();
    let second_of_ann = asked.iter().find(|b| b["messages"].as_array().unwrap().last().unwrap()["content"] == "Q: second of ann").unwrap();
    let history: Vec<&str> = second_of_ann["messages"].as_array().unwrap().iter().skip(1).map(|m| m["content"].as_str().unwrap()).collect();
    assert_eq!(history, ["first of ann", "ok", "Q: second of ann"]);
    // each session has its own file; the one without a name is `default`
    for folder in ["ann", "bob", "default"] {
        assert!(std::path::Path::new("target/test-service/sessions_apart").join(folder).join("memory.json").exists(), "no memory in `{folder}`");
    }
}
