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

/// The sweep deletes only the folders the service made: a folder put in `data_dir` by hand stays, however old, and
/// so does one whose name ends with `~deleting`, as long as it does not hold the mark.
#[tokio::test]
async fn session_foreign_kept() {
    let data = std::path::Path::new("target/test-service/session_foreign_kept");
    let _ = std::fs::remove_dir_all(data);
    std::fs::create_dir_all(data.join("other")).unwrap();
    std::fs::write(data.join("other/notes.txt"), "mine").unwrap();
    std::fs::create_dir_all(data.join("backup~deleting")).unwrap();
    std::fs::write(data.join("backup~deleting/notes.txt"), "mine").unwrap();
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s
        .run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "s5" }))
        .await;
    assert_eq!(status, 200, "{body}");
    assert!(data.join("s5").is_dir());
    assert!(
        wait_until(std::time::Duration::from_secs(10), || !data.join("s5").exists()).await,
        "the session of the service is deleted"
    );
    // the scan that deleted the session may not have reached the other folders yet: one more scan (every second)
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    assert!(
        data.join("other/notes.txt").is_file(),
        "the folder that is not a session stays"
    );
    assert!(
        data.join("backup~deleting/notes.txt").is_file(),
        "the folder `~deleting` without the mark stays"
    );
}
