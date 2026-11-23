use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node calls its tool, answers a text instead of JSON, is corrected, and calls the tool again: the next request
/// keeps the order in which things happened, the correction before the second tool round.
#[tokio::test]
async fn correction_then_tool() {
    let calls = AtomicUsize::new(0);
    let tool = |city: &str| {
        chat(
            json!({ "role": "assistant", "content": "", "tool_calls": [{ "function": { "name": "t", "arguments": { "city": city } } }] }),
        )
    };
    let model = move |_: &Req| match calls.fetch_add(1, Ordering::SeqCst) {
        0 => tool("Roma"),
        1 => chat(json!({ "role": "assistant", "content": "It is sunny in Roma" })),
        2 => tool("Milano"),
        _ => chat(json!({ "role": "assistant", "content": "{\"sunny\": true}" })),
    };
    let (p, seen) = program(here(), "correction_then_tool", model).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(out.items[0].message, Some(Content::Json(json!({ "sunny": true }))));

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 4, "a tool, the text, a tool, the JSON");
    let messages = body_of(&seen[3])["messages"].as_array().unwrap().clone();
    let roles: Vec<&str> = messages.iter().map(|m| m["role"].as_str().unwrap()).collect();
    assert_eq!(
        roles[roles.len() - 6..],
        ["assistant", "tool", "assistant", "user", "assistant", "tool"],
        "{messages:?}"
    );
    assert_eq!(messages[messages.len() - 4]["content"], "It is sunny in Roma");
}
