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

/// Two requests at the same time are both answered.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn together() {
    serve_at(19999, |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path(), 19979).await;
    let (one, two) = tokio::join!(s.run(json!({ "yml": read("graph.yml"), "message": "one" })), s.run(json!({ "yml": read("graph.yml"), "message": "two" })));
    assert_eq!((one.0, two.0), (200, 200));
}
