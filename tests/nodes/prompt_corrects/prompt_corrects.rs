use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An answer that is not one JSON value is sent back to the model, with a fixed request, and the next answer is
/// read again; every attempt is written, and `node.end` sums them.
#[tokio::test]
async fn prompt_corrects() {
    crate::common::clean_output("target/test-output/prompt_corrects.log");
    let (p, seen) = program_seq(
        here(),
        "prompt_corrects",
        vec![chat("The result is six", None), chat("{\"a\": 6}", None)],
    )
    .await;
    let p = Arc::new(p);
    let out = p.run("q").await.unwrap();
    assert_eq!(out.items[0].message, Some(Content::Json(json!({ "a": 6 }))));

    // the second request ends with the wrong answer and the request to correct it
    assert_eq!(seen.lock().unwrap().len(), 2);
    let messages = sent_at(&seen, 1)["messages"].as_array().unwrap().clone();
    let (answer, request) = (&messages[messages.len() - 2], &messages[messages.len() - 1]);
    assert_eq!(
        (answer["role"].as_str(), answer["content"].as_str()),
        (Some("assistant"), Some("The result is six"))
    );
    assert_eq!(request["role"], "user");
    assert!(
        request["content"].as_str().unwrap().contains("one JSON value"),
        "{request}"
    );

    let lines = crate::common::run::log_lines("prompt_corrects");
    let attempts: Vec<&Value> = lines.iter().filter(|l| l["code"] == "structured.attempt").collect();
    let seen_attempts: Vec<(Option<&str>, Option<&str>, Option<u64>, Option<u64>, Option<&str>)> = attempts
        .iter()
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
        seen_attempts,
        [
            (Some("warn"), Some("prompt"), Some(1), Some(3), Some("not_json")),
            (Some("debug"), Some("prompt"), Some(2), Some(3), Some("valid")),
        ]
    );
    assert!(
        attempts[0]["correction"]
            .as_str()
            .is_some_and(|c| c.contains("one JSON value"))
    );
    let end = lines.iter().find(|l| l["code"] == "node.end").unwrap();
    // the mock counts 7 input and 3 output tokens for every call
    assert_eq!(
        (
            end["corrections"].as_u64(),
            end["input_tokens"].as_u64(),
            end["output_tokens"].as_u64()
        ),
        (Some(1), Some(14), Some(6)),
        "{end}"
    );
}
