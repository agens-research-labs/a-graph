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

/// Two requests at the same time are both answered.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn together() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (one, two) = tokio::join!(s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "one" })), s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "two" })));
    assert_eq!((one.0, two.0), (200, 200));
}
