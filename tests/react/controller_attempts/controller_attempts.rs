use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A controller that never answers one JSON value is asked the `max_attempts` of its react, then the run stops
/// (`attempt_limit`), with its last answer as the detail.
#[tokio::test]
async fn controller_attempts() {
    let turns = AtomicUsize::new(0);
    let (p, seen) = program(here(), "controller_attempts", move |req: &Req| {
        let controller = body_of(req)["messages"][0]["content"]
            .as_str()
            .is_some_and(|s| s.contains("next_worker"));
        if !controller {
            return Resp::ok(chat(&user_of(req)));
        }
        let n = turns.fetch_add(1, Ordering::SeqCst);
        Resp::ok(chat(&format!("I call w1, take {n}")))
    })
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("attempt_limit", "node(c)".to_string())
    );
    assert!(!e.message.contains("take"), "{}", e.message);
    assert!(e.detail.as_deref().is_some_and(|d| d.contains("take 1")), "{e:?}");
    // the start node, then the two attempts of the controller
    assert_eq!(seen.lock().unwrap().len(), 3);
}
