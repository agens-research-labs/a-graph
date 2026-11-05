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

/// On `SIGTERM` a run whose write of a board waits for a lock that another process holds is stopped at the 30
/// seconds of the shutdown, and the service exits soon after, with 0: the wait for the lock, left on its thread,
/// does not hold the exit.
#[cfg(unix)]
#[tokio::test]
async fn shutdown_bounded() {
    let session = std::path::Path::new("target/test-service/shutdown_bounded/default");
    std::fs::create_dir_all(session).unwrap();
    std::fs::copy(dir().join("board.md"), session.join("notes.md")).unwrap();
    // the lock of the board, held by the test until it ends: the write of the run waits for it
    let lock = std::fs::File::create(session.join("notes.md.lock")).unwrap();
    lock.lock().unwrap();
    let (model, seen) = listen(|_| Resp::ok(answer("noted"))).await;
    let mut s = start(&config_path()).await;
    let (status, body) = s
        .start_run(json!({ "yml": read("graph.yml", &[("model", model)]), "message": "what?" }))
        .await;
    assert_eq!(status, 202, "{body}");
    assert!(
        wait_until(std::time::Duration::from_secs(10), || seen.lock().unwrap().len() == 1).await,
        "the model was not called"
    );
    let signalled = std::time::Instant::now();
    s.terminate();
    assert_eq!(s.exited(40).await, Some(0));
    assert!(
        signalled.elapsed() < std::time::Duration::from_secs(37),
        "the service took {:?} to exit",
        signalled.elapsed()
    );
    drop(lock);
}
