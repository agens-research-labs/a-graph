pub use serde_json::{Value, json};
pub use a_graph::run::{Content, Program, RunOutput, build};
pub use super::run::{yml_in, yml_with};
pub use super::http::{Req, Resp, listen};

pub fn body_of(req: &Req) -> Value {
    serde_json::from_str(&req.body).unwrap()
}

pub fn chat(message: Value) -> Resp {
    Resp::ok(json!({ "model": "m", "message": message, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string())
}

pub fn user_of(req: &Req) -> String {
    body_of(req)["messages"].as_array().unwrap().iter().rev().find(|m| m["role"] == "user").unwrap()["content"].as_str().unwrap().to_string()
}

/// What the tools gave back, as the model received it: the `tool` messages of the request.
pub fn tool_messages(req: &Req) -> Vec<String> {
    body_of(req)["messages"].as_array().unwrap().iter().filter(|m| m["role"] == "tool").map(|m| m["content"].as_str().unwrap().to_string()).collect()
}

/// The mock model. A request that carries tools asks for `calls` (name and arguments) while the
/// tool has not answered yet (always, if `always`), and then answers `R(<tool messages>)`. A request
/// without tools answers the text it was asked.
pub fn model(calls: Vec<(&'static str, Value)>, always: bool) -> impl Fn(&Req) -> Resp + Send + Sync + 'static {
    move |req| {
        let has_tools = body_of(req).get("tools").is_some();
        let given = tool_messages(req);
        if has_tools && (given.is_empty() || always) {
            let asked: Vec<Value> = calls.iter().map(|(name, args)| json!({ "function": { "name": name, "arguments": args } })).collect();
            chat(json!({ "role": "assistant", "content": "", "tool_calls": asked }))
        } else if has_tools {
            chat(json!({ "role": "assistant", "content": format!("R({})", given.join("|")) }))
        } else {
            chat(json!({ "role": "assistant", "content": user_of(req) }))
        }
    }
}

/// The final output as (agent id, text).
pub fn items(out: &RunOutput) -> Vec<(String, String)> {
    out.items.iter().map(|i| (i.agent_id.clone(), match &i.message { Some(Content::Text(t)) => t.clone(), _ => String::new() })).collect()
}

pub fn city() -> Vec<(&'static str, Value)> {
    vec![("t", json!({ "city": "Roma" }))]
}
