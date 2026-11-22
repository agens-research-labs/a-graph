use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A request of the controller in prompt mode: the routing schema is in its system prompt.
fn prompted(req: &Req) -> bool {
    let first = &body_of(req)["messages"][0];
    first["role"] == "system" && first["content"].as_str().is_some_and(|s| s.contains("next_worker"))
}

/// With `structured_output: prompt` the provider gets no schema; a routing output that is not one JSON value is
/// sent back to the controller, in the turn it belongs to, and the corrections of the turns are summed.
#[tokio::test]
async fn controller_corrects() {
    crate::common::clean_output("target/test-output/controller_corrects.log");
    let turns = AtomicUsize::new(0);
    let (p, seen) = program(here(), "controller_corrects", move |req: &Req| {
        if !prompted(req) {
            return Resp::ok(chat(&user_of(req)));
        }
        let said = match turns.fetch_add(1, Ordering::SeqCst) {
            0 => "Let me call w1".to_string(),
            1 => full(json!({ "done": false, "next_worker": "w1", "input": "go" })).to_string(),
            2 => "Now I am done".to_string(),
            _ => full(json!({ "done": true, "answer": "end" })).to_string(),
        };
        Resp::ok(chat(&said))
    })
    .await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(text(&out, "z"), "Z(end)");

    let asked: Vec<Value> = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| prompted(r))
        .map(body_of)
        .collect();
    assert_eq!(asked.len(), 4);
    assert!(
        asked.iter().all(|b| b.get("format").is_none()),
        "the provider gets no schema"
    );
    let tail = |n: usize| {
        let m = asked[n]["messages"].as_array().unwrap();
        (
            m[m.len() - 2]["content"].as_str().unwrap().to_string(),
            m[m.len() - 1]["content"].as_str().unwrap().to_string(),
        )
    };
    // the corrections close the turn they belong to
    let (wrong, request) = tail(1);
    assert_eq!(wrong, "Let me call w1");
    assert!(request.contains("one JSON value"), "{request}");
    let (wrong, request) = tail(3);
    assert_eq!(wrong, "Now I am done");
    assert!(request.contains("one JSON value"), "{request}");
    // the next turn has the routing output, not the wrong answer before it
    let next = asked[2]["messages"].to_string();
    assert!(!next.contains("Let me call w1"), "{next}");
    assert_eq!(tail(2).1, "Observation from w1: W1(go)");

    let lines = crate::common::run::log_lines("controller_corrects");
    let outcomes: Vec<(Option<u64>, Option<&str>)> = lines
        .iter()
        .filter(|l| l["code"] == "structured.attempt" && l["node"] == "c")
        .map(|a| (a["attempt"].as_u64(), a["outcome"].as_str()))
        .collect();
    assert_eq!(
        outcomes,
        [
            (Some(1), Some("not_json")),
            (Some(2), Some("valid")),
            (Some(1), Some("not_json")),
            (Some(2), Some("valid")),
        ]
    );
    let end = lines
        .iter()
        .find(|l| l["code"] == "node.end" && l["node"] == "c")
        .unwrap();
    // the mock counts one input and one output token at every call
    assert_eq!(
        (end["corrections"].as_u64(), end["input_tokens"].as_u64()),
        (Some(2), Some(4)),
        "{end}"
    );
}
