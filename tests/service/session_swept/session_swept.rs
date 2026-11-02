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

/// With `session_retention_seconds: 1` the folder of a session is deleted when it has been idle for a second; `data_dir` stays.
#[tokio::test]
async fn session_swept() {
    let data = std::path::Path::new("target/test-service/session_swept");
    let _ = std::fs::remove_dir_all(data);
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "s1" }))
        .await;
    assert_eq!(status, 200, "{body}");
    assert!(data.join("s1").is_dir(), "the request makes the folder of its session");
    // the time of the last use is that of the mark, not of the folder
    let stamped = std::fs::metadata(data.join("s1/.a-graph-session"))
        .unwrap()
        .modified()
        .unwrap();
    assert!(
        stamped.elapsed().unwrap() < std::time::Duration::from_secs(5),
        "the request stamps the mark"
    );
    // the folders are looked at every second here (the retention, shorter than 10 seconds)
    assert!(
        wait_until(std::time::Duration::from_secs(10), || !data.join("s1").exists()).await,
        "the idle session is deleted"
    );
    assert!(data.is_dir(), "data_dir itself stays");
    assert_eq!(
        std::fs::read_dir(data).unwrap().count(),
        0,
        "nothing is left of the folder, not even under another name"
    );
}
