use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node calls its tool natively, then answers a text instead of JSON: the correction comes after the tool
/// round, and the next answer is the JSON of the node.
#[tokio::test]
async fn prompt_after_tool() {
    let model = |req: &Req| {
        let messages = body_of(req)["messages"].as_array().unwrap().clone();
        let last = messages.last().unwrap();
        if tool_messages(req).is_empty() {
            chat(
                json!({ "role": "assistant", "content": "", "tool_calls": [{ "function": { "name": "t", "arguments": { "city": "Roma" } } }] }),
            )
        } else if last["role"] == "tool" {
            chat(json!({ "role": "assistant", "content": "It is sunny in Roma" }))
        } else {
            chat(json!({ "role": "assistant", "content": "{\"sunny\": true}" }))
        }
    };
    let (p, seen) = program(here(), "prompt_after_tool", model).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(out.items[0].message, Some(Content::Json(json!({ "sunny": true }))));

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 3, "the tool call, the text, the JSON");
    let messages = body_of(&seen[2])["messages"].as_array().unwrap().clone();
    let roles: Vec<&str> = messages.iter().map(|m| m["role"].as_str().unwrap()).collect();
    assert_eq!(
        roles[roles.len() - 4..],
        ["assistant", "tool", "assistant", "user"],
        "{messages:?}"
    );
    assert_eq!(messages[messages.len() - 2]["content"], "It is sunny in Roma");
}
