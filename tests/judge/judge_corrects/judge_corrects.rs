use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The system prompt of a request: the first message, of role `system`.
fn system_of(req: &Req) -> String {
    let first = &body_of(req)["messages"][0];
    match first["role"].as_str() {
        Some("system") => first["content"].as_str().unwrap_or_default().to_string(),
        _ => String::new(),
    }
}

fn judging(req: &Req) -> bool {
    system_of(req).contains("\"validation\"")
}

/// With `structured_output: prompt` the provider gets no schema: the verdict schema is in the system prompt, and
/// a verdict that is not one JSON value is sent back to the judge.
#[tokio::test]
async fn judge_corrects() {
    crate::common::clean_output("target/test-output/judge_corrects.log");
    let calls = Arc::new(Mutex::new(0usize));
    let counted = calls.clone();
    let (port, seen) = listen(move |req: &Req| {
        if !judging(req) {
            return Resp::ok(chat(&user_of(req)));
        }
        let mut n = counted.lock().unwrap();
        *n += 1;
        Resp::ok(chat(&if *n == 1 {
            "I approve the text".to_string()
        } else {
            approved().to_string()
        }))
    })
    .await;
    let p = Arc::new(build(&yml_with(here(), "judge_corrects", &[("model", port)])).unwrap_or_else(|e| panic!("{e}")));
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out), [pair("z", "Z(S(hi))")]);

    let judge: Vec<Value> = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| judging(r))
        .map(body_of)
        .collect();
    assert_eq!(judge.len(), 2);
    for body in &judge {
        assert!(body.get("format").is_none(), "the provider gets no schema: {body}");
        let system = body["messages"][0]["content"].as_str().unwrap();
        assert!(
            system.starts_with("Judge the text.\n\nAnswer with one JSON value only"),
            "{system}"
        );
    }
    let messages = judge[1]["messages"].as_array().unwrap();
    let (answer, request) = (&messages[messages.len() - 2], &messages[messages.len() - 1]);
    assert_eq!(
        (answer["role"].as_str(), answer["content"].as_str()),
        (Some("assistant"), Some("I approve the text"))
    );
    assert!(
        request["content"].as_str().unwrap().contains("one JSON value"),
        "{request}"
    );

    let lines = crate::common::run::log_lines("judge_corrects");
    let attempts: Vec<(Option<&str>, Option<&str>, Option<u64>, Option<u64>, Option<&str>)> = lines
        .iter()
        .filter(|l| l["code"] == "structured.attempt")
        .map(|a| {
            (
                a["level"].as_str(),
                a["mechanism"].as_str(),
                a["attempt"].as_u64(),
                a["max_attempts"].as_u64(),
                a["outcome"].as_str(),
            )
        })
        .collect();
    assert_eq!(
        attempts,
        [
            (Some("warn"), Some("prompt"), Some(1), Some(3), Some("not_json")),
            (Some("debug"), Some("prompt"), Some(2), Some(3), Some("valid")),
        ]
    );
    let end = lines
        .iter()
        .find(|l| l["code"] == "node.end" && l["node"] == "judge")
        .unwrap();
    // the mock counts one input and one output token at every call
    assert_eq!(
        (
            end["verdict"].as_str(),
            end["corrections"].as_u64(),
            end["input_tokens"].as_u64()
        ),
        (Some("approved"), Some(1), Some(2)),
        "{end}"
    );
}
