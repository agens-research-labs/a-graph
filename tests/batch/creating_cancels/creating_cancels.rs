use crate::common::run_batch::*;
use a_graph::error::{Error, Phase, Rule};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

async fn stopped_while_created(create_after: Duration) -> (Arc<Mutex<Vec<Req>>>, Duration) {
    let (p, seen, _) = program_on(
        here(),
        "creating_cancels",
        Jobs {
            endless: true,
            create_after,
            ..Jobs::default()
        },
    )
    .await;
    let stop = async {
        // s and i answer at once: the request that creates the job is on its way
        tokio::time::sleep(Duration::from_millis(200)).await;
        Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, "stopped")
    };
    let e = p.run_until("hi", stop).await.unwrap_err();
    assert_eq!(e.rule, Rule::RunTimeout);
    let closing = std::time::Instant::now();
    a_graph::run::close(p).await;
    (seen, closing.elapsed())
}

/// A run stopped while the request that creates its batch job is on its way: the request ends, and the job it
/// created is cancelled at once.
#[tokio::test]
async fn creating_cancels() {
    let (seen, _) = stopped_while_created(Duration::from_millis(500)).await;
    assert_eq!(job_sizes(&seen), [2], "the job was created");
    assert_eq!(cancels(&seen), ["/v1/messages/batches/b1/cancel"]);
    let polls = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.method == "GET" && r.path == "/v1/messages/batches/b1")
        .count();
    assert!(polls <= 1, "the job is not polled once its id came: {polls} polls");

    // a creation that does not answer is not waited for past the limit of the cancel, 3 seconds
    let (seen, closing) = stopped_while_created(Duration::from_secs(10)).await;
    assert!(closing < Duration::from_secs(4), "{closing:?}");
    assert!(cancels(&seen).is_empty());
}
