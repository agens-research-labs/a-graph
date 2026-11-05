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

/// A graph that sends a listed variable to a host listed for it runs.
#[tokio::test]
async fn env_allowed() {
    // SAFETY: no other test reads or writes this variable.
    unsafe { std::env::set_var("A_GRAPH_ENV_ALLOWED", "the-secret") };
    let (model, _) = listen(|_| Resp::ok(answer("fine"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "x" }))
        .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["items"][0]["message"], "fine");
}
