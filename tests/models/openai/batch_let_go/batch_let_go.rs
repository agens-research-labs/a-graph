use super::*;
use crate::common::http::listen_delayed;

const CREATE: &str = "/v1/batches";
const CANCEL: &str = "/v1/batches/batch_1/cancel";

/// The mock of the OpenAI batch API, whose answer that creates the job comes `after` late, and which takes a cancel.
async fn mock(after: Duration) -> (String, Arc<Mutex<Vec<Req>>>) {
    let (handler, _) = batch_server(0, "completed");
    let delay = move |r: &Req| {
        if r.method == "POST" && r.path == CREATE {
            after
        } else {
            Duration::ZERO
        }
    };
    let (port, seen) = listen_delayed(delay, move |req: &Req| {
        if req.method == "POST" && req.path == CANCEL {
            Resp::ok("{}")
        } else {
            handler(req)
        }
    })
    .await;
    (format!("http://127.0.0.1:{port}"), seen)
}

fn requests() -> Vec<BatchRequest> {
    vec![BatchRequest {
        id: "one".into(),
        invocation: call("one"),
    }]
}

fn sent(seen: &Arc<Mutex<Vec<Req>>>, method: &str, path: &str) -> usize {
    seen.lock()
        .unwrap()
        .iter()
        .filter(|r| r.method == method && r.path == path)
        .count()
}

/// When nobody waits for the batch any more (the run closed the channel of its events), the provider does not create
/// a job; and when that happens while the job is created, the provider cancels it as soon as it has its id.
#[tokio::test]
async fn batch_let_go() {
    // before the request that creates the job: no job
    let (base, seen) = mock(Duration::ZERO).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    rx.close();
    let r = client(&base).invoke_batch(requests(), tx).await;
    assert!(r.is_err(), "{r:?}");
    assert_eq!(sent(&seen, "POST", CREATE), 0);

    // while the job is created: it is cancelled, and never polled
    let (base, seen) = mock(Duration::from_millis(300)).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let batch = tokio::spawn(async move { client(&base).invoke_batch(requests(), tx).await });
    assert!(matches!(rx.recv().await, Some(BatchEvent::Creating { requests: 1 })));
    rx.close();
    let r = batch.await.unwrap();
    assert!(r.is_err(), "{r:?}");
    assert_eq!(sent(&seen, "POST", CREATE), 1);
    assert_eq!(sent(&seen, "POST", CANCEL), 1);
    let polls = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.method == "GET" && r.path.starts_with(CREATE))
        .count();
    assert_eq!(polls, 0);
}
