use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The controller calls a worker gets its answer and finishes.
#[tokio::test]
async fn one_worker() {
    let (p, seen) = program(
        here(),
        "one_worker",
        script(|n, last| {
            if n == 0 {
                full(json!({ "done": false, "next_worker": "w1", "input": "find X", "reasoning": "I need X" }))
            } else {
                full(json!({ "done": true, "answer": format!("got: {last}") }))
            }
        }),
    )
    .await;
    let out = p.run("hi").await.unwrap();

    // the worker's answer goes to the final output, then the controller's, then `z` runs on it
    assert_eq!(ids(&out), ["s", "w1", "c", "z"]);
    assert_eq!(text(&out, "w1"), "W1(find X)");
    assert_eq!(text(&out, "c"), "got: Observation from w1: W1(find X)");
    assert_eq!(text(&out, "z"), "Z(got: Observation from w1: W1(find X))");

    // what the controller was told: the schema with the names of the workers...
    let turns = controller_turns(&seen);
    assert_eq!(turns.len(), 2);
    assert_eq!(
        turns[0]["format"]["properties"]["next_worker"]["enum"],
        json!(["w1", "wi", ""])
    );
    // ...its first question, and on the second turn the whole conversation so far
    assert_eq!(turns[0]["messages"].as_array().unwrap().len(), 1);
    let second = turns[1]["messages"].as_array().unwrap();
    assert_eq!(
        (second[0]["role"].as_str(), second[0]["content"].as_str()),
        (Some("user"), Some("C(S(hi))"))
    );
    assert_eq!(second[1]["role"], "assistant");
    assert!(second[1]["content"].as_str().unwrap().contains("find X"));
    assert_eq!(
        (second[2]["role"].as_str(), second[2]["content"].as_str()),
        (Some("user"), Some("Observation from w1: W1(find X)"))
    );
}
