use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A react alone, with no `start` and no `end`: its controller takes the message of the user, and the
/// result is what the nodes send to the final output.
#[tokio::test]
async fn react_alone_runs() {
    let (p, seen) = program(
        here(),
        "react_alone_runs",
        script(|n, last| {
            if n == 0 {
                full(json!({ "done": false, "next_worker": "w1", "input": "find X" }))
            } else {
                full(json!({ "done": true, "answer": format!("got: {last}") }))
            }
        }),
    )
    .await;
    let out = p.run("hi").await.unwrap();

    assert_eq!(ids(&out), ["w1", "c"]);
    assert_eq!(text(&out, "w1"), "W1(find X)");
    assert_eq!(text(&out, "c"), "got: Observation from w1: W1(find X)");
    // the first question of the controller is the message of the user, with nothing before it
    let turns = controller_turns(&seen);
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0]["messages"][0]["content"], "C(hi)");
}
