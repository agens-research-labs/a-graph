//! Calling a tool: a program on stdio, a web service on http.

use std::process::Stdio;
use std::time::Duration;

use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use super::ToolOutcome;
use crate::resources::{Tool, ToolTransport};

/// Calls `tool` with the arguments the model chose. `Err` is a call that cannot be made at all (the
/// program does not start, the headers cannot be built): that is a mistake of the configuration,
/// not something the model can fix. Everything the tool itself does wrong is an `Ok` outcome with
/// `is_error`, which the model gets to read.
pub async fn call_tool(tool: &Tool, arguments: &Value) -> Result<ToolOutcome, String> {
    let call = async {
        match &tool.transport {
            ToolTransport::Stdio { command, args } => stdio(command, args, arguments).await,
            ToolTransport::Http { url, headers } => http(url, headers, arguments).await,
        }
    };
    // when the time is up the call is dropped: a program is killed, a request is abandoned
    match tokio::time::timeout(Duration::from_secs(tool.timeout.into()), call).await {
        Ok(outcome) => outcome,
        Err(_) => Ok(ToolOutcome::failed(format!(
            "the tool did not answer within {} s",
            tool.timeout
        ))),
    }
}

pub(crate) async fn stdio(command: &str, args: &[String], arguments: &Value) -> Result<ToolOutcome, String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("cannot start `{command}`: {e}"))?;

    let mut stdin = child.stdin.take().ok_or("the program has no standard input")?;
    let payload = arguments.to_string();
    // the program may answer without reading its input: a closed pipe is not a failure
    let send = async move {
        let _ = stdin.write_all(payload.as_bytes()).await;
        drop(stdin);
    };
    let (_, output) = tokio::join!(send, child.wait_with_output());
    let output = output.map_err(|e| format!("cannot wait for `{command}`: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let plain = format!("the program exited with {}", output.status);
        if stderr.trim().is_empty() {
            return Ok(ToolOutcome::failed(plain));
        }
        return Ok(ToolOutcome::failed_with_data(plain, stderr.trim()));
    }
    Ok(answer(&String::from_utf8_lossy(&output.stdout)))
}

pub(crate) async fn http(
    url: &str,
    headers: &std::collections::BTreeMap<String, String>,
    arguments: &Value,
) -> Result<ToolOutcome, String> {
    let client = crate::net::client()
        .build()
        .map_err(|e| format!("cannot build the HTTP client: {e}"))?;
    let mut request = client.post(url).json(arguments);
    for (name, var) in headers {
        let value = std::env::var(var).map_err(|_| format!("environment variable `{var}` is not set"))?;
        // a value that is not valid is reported by `send`, as before
        request = match crate::net::secret_value(&value) {
            Ok(v) => request.header(name, v),
            Err(_) => request.header(name, value),
        };
    }
    let response = match request.send().await {
        Ok(r) => r,
        Err(e) => return Ok(ToolOutcome::failed(format!("the service did not answer: {e}"))),
    };
    let status = response.status();
    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            return Ok(ToolOutcome::failed(format!(
                "the answer of the service could not be read: {e}"
            )));
        }
    };
    if !status.is_success() {
        return Ok(ToolOutcome::failed_with_data(
            format!("the service answered {status}"),
            format!("the service answered {status}: {}", body.trim()),
        ));
    }
    Ok(answer(&body))
}

/// The text a tool wrote, read as its JSON result.
fn answer(text: &str) -> ToolOutcome {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => ToolOutcome::ok(v),
        // an error of the syntax says where, never what: only the text itself holds data
        Err(e) => ToolOutcome::failed_with_data(
            format!("the tool did not answer with JSON ({e})"),
            format!(
                "the tool did not answer with JSON ({e}); it wrote: {}",
                text.chars().take(200).collect::<String>()
            ),
        ),
    }
}
