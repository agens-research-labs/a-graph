//! The service `a-graph --serve` as a customer uses it: the real executable listening on a port,
//! called over HTTP. The model of a graph is a mock server of the test (`tests/common/http.rs`).

pub use serde_json::{Value, json};
pub use super::http::{Resp, listen, listen_slowly};
pub use super::with_ports;

/// A running service; it is killed when this is dropped.
pub struct Service {
    _child: tokio::process::Child,
    pub port: u16,
}

/// Starts the service with the config file, on a free port (its `{{port:service}}`), and waits until it
/// answers `/health`. The port is free when it is chosen; if another program takes it before the
/// service does, the service ends and is started again on another one.
pub async fn start(config: &str) -> Service {
    start_with(config, &[]).await
}

/// Like `start`, with the ports of the mock servers the config names too.
pub async fn start_with(config: &str, ports: &[(&str, u16)]) -> Service {
    let text = std::fs::read_to_string(config).unwrap_or_else(|e| panic!("cannot read {config}: {e}"));
    let path = std::path::Path::new(config);
    let stem = format!("service-{}-{}", path.parent().unwrap().file_name().unwrap().to_string_lossy(), path.file_stem().unwrap().to_string_lossy());
    for _ in 0..5 {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let file = super::ported_file(&stem, &with_ports(&text, &[ports, &[("service", port)]].concat()));
        let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_a-graph"))
            .args(["--serve", &file])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("cannot start a-graph");
        for _ in 0..100 {
            if reqwest::get(format!("http://127.0.0.1:{port}/health")).await.is_ok() {
                return Service { _child: child, port };
            }
            if child.try_wait().unwrap().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
    panic!("the service of {config} did not start");
}

impl Service {
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
