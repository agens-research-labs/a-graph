use crate::common::cli::*;
use crate::common::toolmock::model;
use std::time::{Duration, Instant};

fn path() -> String {
    std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join(std::path::Path::new(file!()).file_stem().unwrap())
        .with_extension("yml")
        .to_string_lossy()
        .to_string()
}

/// Runs the graph, sends Ctrl-C `times` times while its tool is busy, 300 ms apart, and gives what it wrote on the
/// standard error, its exit code and the time from the first Ctrl-C to its exit.
async fn interrupted(times: usize) -> (String, Option<i32>, Duration) {
    crate::common::ensure_mcp_server();
    let (port, seen) = listen(model(vec![("slow", json!({}))], false)).await;
    let child = tokio::process::Command::new(env!("CARGO_BIN_EXE_a-graph"))
        .args([&ported(&path(), &[("model", port)]), "go"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("cannot start a-graph");
    let waiting = Instant::now();
    while seen.lock().unwrap().is_empty() {
        assert!(
            waiting.elapsed() < Duration::from_secs(10),
            "the model was never called"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // the model asked for the tool: give the call the time to reach the server
    tokio::time::sleep(Duration::from_millis(300)).await;
    let pid = child.id().unwrap().to_string();
    let first = Instant::now();
    for i in 0..times {
        if i > 0 {
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        assert!(
            std::process::Command::new("kill")
                .args(["-INT", &pid])
                .status()
                .unwrap()
                .success()
        );
    }
    // the end of a-graph is timed on its exit: the MCP server it started may hold the standard error a little longer
    let mut child = child;
    let mut err = child.stderr.take().unwrap();
    let read = tokio::spawn(async move {
        let mut text = String::new();
        tokio::io::AsyncReadExt::read_to_string(&mut err, &mut text)
            .await
            .unwrap();
        text
    });
    let status = child.wait().await.unwrap();
    let took = first.elapsed();
    (read.await.unwrap(), status.code(), took)
}

/// A second Ctrl-C ends the process at once: it does not wait for the run to close, here for the MCP server whose
/// tool is busy, which is let go after two seconds.
#[cfg(unix)]
#[tokio::test]
async fn ctrl_c_twice() {
    // one Ctrl-C: the run closes, and that takes the two seconds of the busy server
    let (stderr, code, took) = interrupted(1).await;
    assert_eq!(code, Some(3), "{stderr}");
    assert!(stderr.contains("(interrupted)"), "{stderr}");
    assert!(took >= Duration::from_millis(1800), "{took:?}");

    // two Ctrl-C: the process ends at the second, before the run closed and wrote its error
    let (stderr, code, took) = interrupted(2).await;
    assert_eq!(code, Some(3), "{stderr}");
    assert!(
        stderr.contains("stopping the run, Ctrl-C again to quit at once"),
        "{stderr}"
    );
    assert!(!stderr.contains("(interrupted)"), "{stderr}");
    assert!(took < Duration::from_millis(1500), "{took:?}");
}
