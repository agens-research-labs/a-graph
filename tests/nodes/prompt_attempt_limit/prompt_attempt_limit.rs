use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model that never answers one JSON value is asked `max_attempts` times, then the node stops
/// (`attempt_limit`), with its last answer as the detail.
#[tokio::test]
async fn prompt_attempt_limit() {
    let (p, seen) = program_seq(
        here(),
        "prompt_attempt_limit",
        vec![chat("six", None), chat("still six", None)],
    )
    .await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "attempt_limit");
    assert!(!e.message.contains("six"), "{}", e.message);
    assert!(e.detail.as_deref().is_some_and(|d| d.contains("still six")), "{e:?}");
    assert_eq!(seen.lock().unwrap().len(), 2);
}
