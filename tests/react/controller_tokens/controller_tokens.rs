use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The end of the controller counts the tokens of all its turns, not of the last one.
#[tokio::test]
async fn controller_tokens() {
    crate::common::clean_output("target/test-output/controller_tokens.log");
    let (p, seen) = program(
        here(),
        "controller_tokens",
        script(|n, _| match n {
            0 | 1 => full(json!({ "done": false, "next_worker": "w1", "input": "go" })),
            _ => full(json!({ "done": true, "answer": "end" })),
        }),
    )
    .await;
    p.run("hi").await.unwrap();
    assert_eq!(controller_turns(&seen).len(), 3);
    let lines = crate::common::run::log_lines("controller_tokens");
    let end = lines
        .iter()
        .find(|l| l["code"] == "node.end" && l["node"] == "c")
        .unwrap();
    // the mock counts one input and one output token at every call
    assert_eq!(
        (end["input_tokens"].as_u64(), end["output_tokens"].as_u64()),
        (Some(3), Some(3))
    );
}
