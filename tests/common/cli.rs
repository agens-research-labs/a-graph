//! The program `a-graph` as a user runs it: the real executable, with arguments and a standard
//! input, and what it wrote and the code it ended with. The model of a test is a mock server in the
//! same test (`tests/common/http.rs`), reached by the executable over the network as a real one.

pub use serde_json::{Value, json};
pub use super::http::{Resp, serve_at};

pub struct Ran {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runs `a-graph` with these arguments. `stdin` is what it receives on the standard input
/// (nothing, if `None`: the input is closed). The path of a yml is relative to the project.
pub async fn a_graph(args: &[&str], stdin: Option<&str>) -> Ran {
    use tokio::io::AsyncWriteExt;
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_a-graph"))
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("cannot start a-graph");
    let mut input = child.stdin.take().unwrap();
    if let Some(text) = stdin {
        input.write_all(text.as_bytes()).await.unwrap();
    }
    drop(input);
    let out = child.wait_with_output().await.unwrap();
    Ran { code: out.status.code().unwrap_or(-1), stdout: String::from_utf8_lossy(&out.stdout).to_string(), stderr: String::from_utf8_lossy(&out.stderr).to_string() }
}

/// The answer of a mock Ollama model: the text.
pub fn answer(text: &str) -> String {
    json!({ "model": "m", "message": { "role": "assistant", "content": text }, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string()
}
