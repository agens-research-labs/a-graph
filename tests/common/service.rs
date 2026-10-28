//! The service `a-graph --serve` as a customer uses it: the real executable listening on a port,
//! called over HTTP. The model of a graph is a mock server of the test (`tests/common/http.rs`).

pub use serde_json::{Value, json};
pub use super::http::{Resp, serve_at, serve_at_slowly};

/// A running service; it is killed when this is dropped.
pub struct Service {
    child: tokio::process::Child,
    pub port: u16,
}

/// Starts the service with the config file and waits until it answers `/health`.
pub async fn start(config: &str, port: u16) -> Service {
    let child = tokio::process::Command::new(env!("CARGO_BIN_EXE_a-graph"))
        .args(["--serve", config])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("cannot start a-graph");
    for _ in 0..100 {
        if reqwest::get(format!("http://127.0.0.1:{port}/health")).await.is_ok() {
            return Service { child, port };
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("the service did not start on port {port}");
}

impl Service {
    /// Sends `SIGTERM` to the service, as `docker stop` does.
    #[cfg(unix)]
    pub fn terminate(&self) {
        let pid = self.child.id().expect("the service already ended").to_string();
        let status = std::process::Command::new("kill").args(["-TERM", &pid]).status().expect("cannot run kill");
        assert!(status.success(), "kill -TERM {pid} failed");
    }

    /// Waits at most `seconds` for the service to end, and gives its exit code.
    pub async fn exited(&mut self, seconds: u64) -> Option<i32> {
        let status = tokio::time::timeout(std::time::Duration::from_secs(seconds), self.child.wait()).await.expect("the service did not end in time").unwrap();
        status.code()
    }

    /// GET this path: the status and the JSON.
    pub async fn get(&self, path: &str) -> (u16, Value) {
        let r = reqwest::get(format!("http://127.0.0.1:{}{path}", self.port)).await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    /// POST /runs: the run is started; the status and the JSON (`{ "run": id }`).
    pub async fn start_run(&self, body: Value) -> (u16, Value) {
        let r = reqwest::Client::new().post(format!("http://127.0.0.1:{}/runs", self.port)).json(&body).send().await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    /// GET the events of a run and read them to the end (the stream closes after `result` or `error`).
    pub async fn events(&self, run: &str) -> Vec<(String, Value)> {
        let text = reqwest::get(format!("http://127.0.0.1:{}/runs/{run}/events", self.port)).await.unwrap().text().await.unwrap();
        parse_sse(&text)
    }

    /// Waits until the run is no longer running and gives its state.
    pub async fn finished(&self, run: &str) -> Value {
        for _ in 0..200 {
            let (_, state) = self.get(&format!("/runs/{run}")).await;
            if state["state"] != "running" {
                return state;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        panic!("the run {run} did not end");
    }

    /// POST /run with this JSON body: the status and the JSON that came back.
    pub async fn run(&self, body: Value) -> (u16, Value) {
        let r = reqwest::Client::new().post(format!("http://127.0.0.1:{}/run", self.port)).json(&body).send().await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    /// POST /run with this text as the body, whatever it is.
    pub async fn run_raw(&self, body: &str) -> (u16, Value) {
        let r = reqwest::Client::new().post(format!("http://127.0.0.1:{}/run", self.port)).body(body.to_string()).send().await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }
}

/// The events of an SSE text, as (name, data).
pub fn parse_sse(text: &str) -> Vec<(String, Value)> {
    text.split("\n\n")
        .filter_map(|block| {
            let name = block.lines().find_map(|l| l.strip_prefix("event:"))?.trim().to_string();
            let data = block.lines().find_map(|l| l.strip_prefix("data:"))?.trim();
            Some((name, serde_json::from_str(data).unwrap_or(Value::Null)))
        })
        .collect()
}

/// The answer of a mock Ollama model: the text.
pub fn answer(text: &str) -> String {
    json!({ "model": "m", "message": { "role": "assistant", "content": text }, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string()
}
