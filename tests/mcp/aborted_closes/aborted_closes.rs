use crate::common::run_mcp::*;
use a_graph::error::{Error, Path, Phase, Rule};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A run stopped while a branch of a `parallel` group calls a tool of an MCP server closes the servers before it
/// ends, though the aborted branch lets go of them only a moment later. The busy server may be let go after two
/// seconds; the idle one is closed and its process has ended when the run ends.
#[tokio::test]
async fn aborted_closes() {
    let pid_file = "target/test-output/aborted_closes.pid";
    std::fs::create_dir_all("target/test-output").unwrap();
    let _ = std::fs::remove_file(pid_file);
    let (p, seen) = program(here(), "aborted_closes", model(ask("slow", json!({})), false)).await;
    // the stop comes once a asked for the tool, so its branch is in the call: a fixed time could come before
    let stop = async move {
        while !seen.lock().unwrap().iter().any(|r| body_of(r).get("tools").is_some()) {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, "stopped")
    };
    let e = p.run_until("go", stop).await.unwrap_err();
    assert_eq!(e.rule, Rule::RunTimeout);
    let pid = std::fs::read_to_string(pid_file).expect("the server wrote its pid");
    // `kill -0` fails when the process is gone: the idle server was closed before the run ended
    let alive = std::process::Command::new("kill")
        .args(["-0", pid.trim()])
        .status()
        .unwrap()
        .success();
    assert!(!alive, "the idle server {pid} is still running");
}
