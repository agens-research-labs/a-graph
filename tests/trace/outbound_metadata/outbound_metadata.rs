use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Observe the actual REST request, including HTTP success followed by a parse failure.
/// Userinfo, paths and query strings must never enter metadata, logs or exported spans.
#[tokio::test]
async fn outbound_metadata() {
    for (status, body, success) in [
        (200, chat("ok"), true),
        (401, "refused".into(), false),
        (200, "not JSON".into(), false),
    ] {
        let (port, collector) = listen(|_| Resp::ok("{}")).await;
        let (model_port, requests) = listen(move |_| Resp::status(status, body.clone())).await;
        let source = yml_with(
            here(),
            "outbound_metadata",
            &[("collector", port), ("model", model_port)],
        );
        let mut p = Arc::new(build(&source).unwrap());
        let events = Arc::new(Mutex::new(Vec::new()));
        let observed = events.clone();
        Arc::get_mut(&mut p).unwrap().observer =
            Some(Arc::new(move |event| observed.lock().unwrap().push(event.clone())));
        assert_eq!(p.run("hi").await.is_ok(), success);
        let spans = spans_of(&collector);
        let call = named(&spans, "model m");
        for (key, value) in [
            ("server.address", json!({"stringValue": "127.0.0.1"})),
            ("server.port", json!({"intValue": model_port.to_string()})),
            ("url.scheme", json!({"stringValue": "http"})),
            ("http.request.method", json!({"stringValue": "POST"})),
            ("http.response.status_code", json!({"intValue": status.to_string()})),
        ] {
            assert_eq!(attribute(call, key), Some(value), "{key}");
        }
        assert!(
            attribute(call, "a_graph.http_request_count").is_none(),
            "one REST request needs no count annotation"
        );
        let log = events.lock().unwrap();
        let completion = log
            .iter()
            .find(|e| e.code == if success { "model.call" } else { "model.error" })
            .unwrap();
        assert!(
            completion
                .fields
                .iter()
                .any(|(k, v)| k == "http.response.status_code" && v == &json!(status))
        );
        let sent = collector.lock().unwrap();
        for sentinel in [
            "DO_NOT_EXPORT_USER",
            "DO_NOT_EXPORT_PASSWORD",
            "DO_NOT_EXPORT_PATH",
            "DO_NOT_EXPORT_QUERY",
        ] {
            assert!(!sent[0].body.contains(sentinel));
            assert!(!format!("{log:?}").contains(sentinel));
        }
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, "POST");
        assert!(
            !requests[0].headers.contains("traceparent:"),
            "propagation is a separate feature"
        );
    }
}

/// More than one REST request is explicitly annotated; do not overwrite the primary request
/// with metadata from a later download or follow-up request.
#[tokio::test]
async fn multiple_requests() {
    use a_graph::models::{Model, Ollama};
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let (first_port, _) = listen(echo).await;
    let (second_port, _) = listen(echo).await;
    let source = yml_with(
        here(),
        "outbound_metadata",
        &[("collector", port), ("model", first_port)],
    );
    let mut p = Arc::new(build(&source).unwrap());
    let Model::Ollama(first) = p.models.spec("m").unwrap().clone() else {
        panic!("expected Ollama")
    };
    let mut second = first.clone();
    second.host = format!("http://127.0.0.1:{second_port}");
    let model = TwoRequests(
        Ollama::ollama(&first, &a_graph::error::Path::root()).unwrap(),
        Ollama::ollama(&second, &a_graph::error::Path::root()).unwrap(),
    );
    Arc::get_mut(&mut p).unwrap().models.insert("m", Box::new(model));
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    let call = named(&spans, "model m");
    assert_eq!(
        attribute(call, "a_graph.http_request_count"),
        Some(json!({"intValue": "2"}))
    );
    assert_eq!(
        attribute(call, "server.port"),
        Some(json!({"intValue": first_port.to_string()}))
    );
    assert_eq!(
        attribute(call, "http.response.status_code"),
        Some(json!({"intValue": "200"}))
    );
}

struct TwoRequests(a_graph::models::Ollama, a_graph::models::Ollama);

#[async_trait::async_trait]
impl Llm for TwoRequests {
    fn check(&self, requirements: &ModelRequirements) -> Result<(), ModelError> {
        self.0.check(requirements)
    }
    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        self.0.invoke(call).await?;
        self.1.invoke(call).await
    }
    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        self.0.invoke_batch(requests, events).await
    }
}

/// Network failures preserve the attempted destination/method without inventing a response status.
#[tokio::test]
async fn network_failure() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let model_port = listener.local_addr().unwrap().port();
    let peer = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        drop(socket);
        drop(listener);
    });
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let source = yml_with(
        here(),
        "outbound_metadata",
        &[("collector", port), ("model", model_port)],
    );
    let p = Arc::new(build(&source).unwrap());
    let error = tokio::time::timeout(Duration::from_secs(5), p.run("hi"))
        .await
        .unwrap()
        .unwrap_err();
    for sentinel in [
        "DO_NOT_EXPORT_USER",
        "DO_NOT_EXPORT_PASSWORD",
        "DO_NOT_EXPORT_PATH",
        "DO_NOT_EXPORT_QUERY",
    ] {
        assert!(!error.to_string().contains(sentinel));
    }
    peer.await.unwrap();
    let spans = spans_of(&collector);
    let call = named(&spans, "model m");
    assert_eq!(attribute(call, "error.type"), Some(json!({"stringValue": "network"})));
    assert_eq!(
        attribute(call, "server.port"),
        Some(json!({"intValue": model_port.to_string()}))
    );
    assert_eq!(
        attribute(call, "http.request.method"),
        Some(json!({"stringValue": "POST"}))
    );
    assert!(attribute(call, "http.response.status_code").is_none());
    let sent = collector.lock().unwrap();
    assert!(
        !sent[0].body.contains("DO_NOT_EXPORT"),
        "including root span error status"
    );
}
