use crate::common::run_mcp::*;
use a_graph::error::{Error, Path, Phase, Rule};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A run stopped while a tool of an MCP server is called ends with the error of the stop, and still ends as
/// always: the server is closed (it is let go after two seconds at most, though busy) and the last event is logged.
#[tokio::test]
async fn stopped_closes() {
    crate::common::clean_output("target/test-output/stopped_closes.log");
    let (p, _) = program(here(), "stopped_closes", model(ask("slow", json!({})), false)).await;
    let started = std::time::Instant::now();
    let stop = async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, "stopped")
    };
    let e = p.run_until("go", stop).await.unwrap_err();
    assert_eq!(e.rule, Rule::RunTimeout);
    // the tool takes ten seconds: the run does not wait for it
    assert!(started.elapsed() < std::time::Duration::from_secs(6), "{:?}", started.elapsed());
    let lines = crate::common::run::log_lines("stopped_closes");
    assert_eq!(lines.last().map(|l| &l["code"]), Some(&json!("run_timeout")), "{lines:?}");
}
