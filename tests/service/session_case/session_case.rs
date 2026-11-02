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

/// The names of the folders in `data`.
fn folders(data: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(data).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
    names.sort();
    names
}

/// A name keeps its case: `ABC` and `abc` have two folders, unless the filesystem ignores case, where they are one.
/// A run on `ABC` keeps `abc` from the sweep too, since on such a filesystem it is the same folder.
#[tokio::test]
async fn session_case() {
    let data = std::path::Path::new("target/test-service/session_case");
    let _ = std::fs::remove_dir_all(data);
    let (model, _) = listen_slowly(std::time::Duration::from_secs(4), |_| Resp::ok(answer("ok"))).await;
    let s = start(&config_path()).await;
    let (status, body) = s.run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "abc" })).await;
    assert_eq!(status, 200, "{body}");
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "hi", "session": "ABC" })).await;
    assert_eq!(status, 202, "{body}");
    let id = body["run"].as_str().unwrap().to_string();
    // the retention (1 s) plus the scan interval (1 s) plus a margin: the sweep has looked at `abc` while the run on
    // `ABC` goes on
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    assert_eq!(s.get(&format!("/runs/{id}")).await.1["state"], "running", "the run is still going on");
    let names = folders(data);
    assert!(names == ["ABC", "abc"] || names == ["abc"], "a folder for each name (one, where the filesystem ignores case), kept while the run uses it: {names:?}");
    assert_eq!(s.finished(&id).await["state"], "done");
}
