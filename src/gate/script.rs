//! Running a script of a skill: a file of its `scripts/` folder, started as it is.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use serde_json::Value;
use tokio::process::Command;

use super::ToolOutcome;

/// Runs `script` with `args`, in the folder of the skill, and gives back what it wrote. No shell is
/// used: the arguments are a list and nothing is expanded. The script does not get the environment
/// of the program, which has the keys of the models: only `PATH`. It has no standard input.
/// Longer than `timeout` seconds, it is stopped. What it writes is cut at `max_bytes`.
pub async fn run_script(folder: &str, script: &Path, args: &[String], timeout: u32, max_bytes: u64) -> ToolOutcome {
    let mut command = Command::new(script);
    command
        .args(args)
        .current_dir(folder)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Ok(path) = std::env::var("PATH") {
        command.env("PATH", path);
    }
    let name = script
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let child = match command.spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return ToolOutcome::failed(format!("`{name}` cannot be run: it has no permission to execute"));
        }
        Err(e) => return ToolOutcome::failed(format!("`{name}` cannot be run: {e}")),
    };
    let output = match tokio::time::timeout(Duration::from_secs(timeout.into()), child.wait_with_output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return ToolOutcome::failed(format!("`{name}` failed: {e}")),
        Err(_) => return ToolOutcome::failed(format!("`{name}` did not finish within {timeout} s")),
    };
    if !output.status.success() {
        let stderr = cut(&output.stderr, max_bytes);
        return ToolOutcome::failed(if stderr.trim().is_empty() {
            format!("`{name}` exited with {}", output.status)
        } else {
            stderr.trim().to_string()
        });
    }
    ToolOutcome::ok(Value::String(cut(&output.stdout, max_bytes)))
}

/// The text of `bytes`, cut at `max` bytes (on a character) with a note.
fn cut(bytes: &[u8], max: u64) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.len() as u64 <= max {
        return text.into_owned();
    }
    let mut end = max as usize;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n[output cut at {max} bytes]", &text[..end])
}
