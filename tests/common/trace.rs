//! The traces a run sends: the collector is a mock server (`tests/common/http.rs`) that keeps the
//! requests, and these helpers read the spans out of them.

use super::http::Req;
pub use serde_json::{Value, json};

/// Every span of every POST the collector received.
pub fn spans_of(seen: &std::sync::Arc<std::sync::Mutex<Vec<Req>>>) -> Vec<Value> {
    let mut out = Vec::new();
    for r in seen.lock().unwrap().iter() {
        let body: Value =
            serde_json::from_str(&r.body).unwrap_or_else(|e| panic!("the body is not JSON: {e}\n{}", r.body));
        for rs in body["resourceSpans"].as_array().unwrap() {
            for ss in rs["scopeSpans"].as_array().unwrap() {
                out.extend(ss["spans"].as_array().unwrap().iter().cloned());
            }
        }
    }
    out
}

/// The span with this name; the first one if there are several.
pub fn named<'a>(spans: &'a [Value], name: &str) -> &'a Value {
    spans.iter().find(|s| s["name"] == name).unwrap_or_else(|| {
        panic!(
            "no span `{name}` in {:?}",
            spans.iter().map(|s| s["name"].clone()).collect::<Vec<_>>()
        )
    })
}

pub fn start(s: &Value) -> u64 {
    s["startTimeUnixNano"].as_str().unwrap().parse().unwrap()
}

pub fn end(s: &Value) -> u64 {
    s["endTimeUnixNano"].as_str().unwrap().parse().unwrap()
}

/// The attribute `key` of a span as JSON.
pub fn attribute(s: &Value, key: &str) -> Option<Value> {
    s["attributes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["key"] == key)
        .map(|a| a["value"].clone())
}
