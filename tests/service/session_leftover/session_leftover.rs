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

/// A folder whose deletion was cut short (`<name>~deleting`) is deleted by the next scan, even without the mark;
/// a session cannot have that name, since `~` is not allowed in it.
#[tokio::test]
async fn session_leftover() {
    let data = std::path::Path::new("target/test-service/session_leftover");
    let _ = std::fs::remove_dir_all(data);
    std::fs::create_dir_all(data.join("x~deleting/inner")).unwrap();
    std::fs::write(data.join("x~deleting/inner/half.txt"), "left").unwrap();
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "x~deleting" })).await;
    assert_eq!(status, 400, "a session name with `~` is refused: {body}");
    let gone = wait_until(std::time::Duration::from_secs(10), || !data.join("x~deleting").exists()).await;
    assert!(gone, "the folder left halfway is deleted");
    assert!(data.is_dir(), "data_dir itself stays");
}
