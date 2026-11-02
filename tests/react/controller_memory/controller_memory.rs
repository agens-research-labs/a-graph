use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The history the controller reads from its memory stays in every turn: the steps of the react
/// come after it, they do not replace it.
#[tokio::test]
async fn controller_memory() {
    let (p, seen) = program(
        here(),
        "controller_memory",
        script(|n, _| {
            if n == 0 {
                json!({ "done": false, "next_worker": "w1", "input": "find X" })
            } else {
                json!({ "done": true, "answer": "end" })
            }
        }),
    )
    .await;
    p.run("hi").await.unwrap();

    let turns = controller_turns(&seen);
    assert_eq!(turns.len(), 2);
    let pairs = |turn: &Value| -> Vec<(String, String)> {
        turn["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                (
                    m["role"].as_str().unwrap().to_string(),
                    m["content"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    };

    // the first turn: the memory, then the question
    let first = pairs(&turns[0]);
    assert_eq!(first.len(), 3);
    assert_eq!(
        first[0],
        ("user".to_string(), "I am Anna and I am vegetarian".to_string())
    );
    assert_eq!(
        first[1],
        ("assistant".to_string(), "Nice to meet you, Anna".to_string())
    );
    assert_eq!(first[2], ("user".to_string(), "C(S(hi))".to_string()));

    // the second turn: the same memory, then the question, the decision and the observation
    let second = pairs(&turns[1]);
    assert_eq!(second.len(), 5);
    assert_eq!(second[0], first[0]);
    assert_eq!(second[1], first[1]);
    assert_eq!(second[2], first[2]);
    assert_eq!(second[3].0, "assistant");
    assert!(second[3].1.contains("find X"));
    assert_eq!(
        second[4],
        ("user".to_string(), "Observation from w1: W1(find X)".to_string())
    );
}
