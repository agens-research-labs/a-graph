use crate::common::run_batch::*;
use a_graph::error::{Error, Phase, Rule};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A run stopped while it waits for its batch job asks the provider to cancel the job, and the end of the
/// program waits for that cancel.
#[tokio::test]
async fn stopped_cancels() {
    let (p, seen, _) = program_on(
        here(),
        "stopped_cancels",
        Jobs {
            endless: true,
            ..Jobs::default()
        },
    )
    .await;
    let stop = async {
        // the job is created and polled several times in the meantime
        tokio::time::sleep(Duration::from_millis(300)).await;
        Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, "stopped")
    };
    let e = p.run_until("hi", stop).await.unwrap_err();
    assert_eq!(e.rule, Rule::RunTimeout);
    a_graph::run::close(p).await;

    assert_eq!(job_sizes(&seen), [2], "one job with the requests of d and e");
    assert_eq!(cancels(&seen), ["/v1/messages/batches/b1/cancel"]);
}
