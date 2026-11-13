use super::*;
use std::time::Duration;

/// A result file that arrives too slowly ends the batch at the call timeout, as the call that asked for it
/// would: the SDK bounds the request up to its headers, not the body that follows.
#[tokio::test]
async fn batch_slow_file() {
    let (handler, _) = batch_server(0, "Completed", true);
    let (base, _) = serve(move |req: &Req| {
        let resp = handler(req);
        if req.path.starts_with("/my-out/results/job42/x.jsonl.out") {
            // about 20 bytes a second: never stalled, so the SDK keeps reading
            resp.dripping(Duration::from_millis(50))
        } else {
            resp
        }
    })
    .await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let bedrock = batch_client(&base).with_call_timeout(Duration::from_secs(1));
    let requests = vec![BatchRequest {
        id: "one".into(),
        invocation: call("one"),
    }];

    let r = tokio::time::timeout(Duration::from_secs(10), bedrock.invoke_batch(requests, tx))
        .await
        .expect("the batch must end by itself at the call timeout");
    assert!(
        matches!(&r, Err(ModelError::Network { .. })) && format!("{r:?}").contains("result file"),
        "{r:?}"
    );
}
