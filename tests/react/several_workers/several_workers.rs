use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The controller can call several workers in turn.
#[tokio::test]
async fn several_workers() {
    let (p, seen) = program(
        here(),
        "several_workers",
        script(|n, last| match n {
            0 => full(json!({ "done": false, "next_worker": "w1", "input": "one" })),
            1 => full(json!({ "done": false, "next_worker": "w1", "input": format!("two after {last}") })),
            _ => full(json!({ "done": true, "answer": "end" })),
        }),
    )
    .await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(ids(&out), ["s", "w1", "w1", "c", "z"]);
    assert_eq!(controller_turns(&seen).len(), 3);
}
