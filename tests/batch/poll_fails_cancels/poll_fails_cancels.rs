use crate::common::run_batch::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A batch whose poll fails 10 times in a row ends the run with the error of the provider, and its job, which the
/// run no longer follows, is cancelled at the provider.
#[tokio::test]
async fn poll_fails_cancels() {
    let (p, seen, _) = program_on(
        here(),
        "poll_fails_cancels",
        Jobs {
            poll_fails: true,
            ..Jobs::default()
        },
    )
    .await;
    let run = tokio::time::timeout(Duration::from_secs(10), p.run("hi")).await;
    let e = run.expect("the failed polls end the run").unwrap_err();
    assert!(e.message.contains("529"), "{e:?}");
    a_graph::run::close(p).await;

    let polls = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.method == "GET" && r.path == "/v1/messages/batches/b1")
        .count();
    assert_eq!(polls, 10);
    assert_eq!(cancels(&seen), ["/v1/messages/batches/b1/cancel"]);
}
