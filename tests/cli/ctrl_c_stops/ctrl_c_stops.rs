use crate::common::cli::*;
use crate::common::http::listen_slowly;
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

/// Ctrl-C (`SIGINT`) during a run stops it as a run that lasted too long is stopped: the run ends as failed with
/// the rule `interrupted`, exit code 3, at once and not when the model answers.
#[cfg(unix)]
#[tokio::test]
async fn ctrl_c_stops() {
    let (model, seen) = listen_slowly(Duration::from_secs(30), |_| Resp::ok(answer("late"))).await;
    let child = tokio::process::Command::new(env!("CARGO_BIN_EXE_a-graph"))
        .args([&ported(&path(), &[("model", model)]), "hi"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("cannot start a-graph");
    // the first node waits for the model: the handler is set before the run starts
    let waiting = Instant::now();
    while seen.lock().unwrap().is_empty() {
        assert!(
            waiting.elapsed() < Duration::from_secs(10),
            "the model was never called"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let sent = Instant::now();
    let pid = child.id().unwrap().to_string();
    let killed = std::process::Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .unwrap();
    assert!(killed.success());

    let out = child.wait_with_output().await.unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(3), "{stderr}");
    assert!(sent.elapsed() < Duration::from_secs(5), "{:?}", sent.elapsed());
    assert!(
        stderr.contains("stopping the run, Ctrl-C again to quit at once"),
        "{stderr}"
    );
    assert!(
        stderr.contains("(interrupted): the run was interrupted (Ctrl-C)"),
        "{stderr}"
    );
    assert!(out.stdout.is_empty());
}
