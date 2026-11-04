use crate::common::run_batch::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A `batch` group that is a branch of a `parallel` group: when the other branch fails, the run ends with its
/// error, and the job of the batch, which would never end, is cancelled at the provider.
#[tokio::test]
async fn failed_branch_cancels() {
    let (p, seen, _) = program_on(
        here(),
        "failed_branch_cancels",
        Jobs {
            endless: true,
            refuse_call: Some("X(".into()),
            ..Jobs::default()
        },
    )
    .await;
    // x is refused `REFUSE_AFTER` late: by then b has answered and the job is created and polled
    let run = tokio::time::timeout(Duration::from_secs(10), p.run("hi")).await;
    let e = run.expect("the failure of x ends the run").unwrap_err();
    assert!(e.message.contains("answered 500"), "{e:?}");
    a_graph::run::close(p).await;

    assert_eq!(cancels(&seen), ["/v1/messages/batches/b1/cancel"]);
}
