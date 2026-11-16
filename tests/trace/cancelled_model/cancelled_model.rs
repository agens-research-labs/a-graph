use crate::common::run_graph::*;
use crate::common::trace::*;
use a_graph::error::{Error, Path, Phase, Rule};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Dropping an in-flight provider future must not erase the attempted call from the trace.
#[tokio::test]
async fn cancelled_model() {
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let (mut p, _) = program_with(here(), "cancelled_model", &[("collector", port)], echo).await;
    let started = Arc::new(tokio::sync::Notify::new());
    let notify = started.clone();
    Arc::get_mut(&mut p).unwrap().observer = Some(Arc::new(move |ev| {
        if ev.code == "model.start" {
            notify.notify_one();
        }
    }));
    let stop = async {
        started.notified().await;
        Error::new(Phase::Execution, Path::root(), Rule::ModelFailed, "test cancellation")
    };
    assert!(p.run_until("hi", stop).await.is_err());
    let spans = spans_of(&collector);
    let call = named(&spans, "model m");
    assert_eq!(call["kind"], 3);
    assert_eq!(
        attribute(call, "gen_ai.provider.name"),
        Some(json!({"stringValue": "ollama"}))
    );
    assert_eq!(
        attribute(call, "gen_ai.request.model"),
        Some(json!({"stringValue": "m"}))
    );
    assert_eq!(call["status"]["code"], 2);
    assert_eq!(attribute(call, "error.type"), Some(json!({"stringValue": "cancelled"})));
    assert_eq!(call["parentSpanId"], named(&spans, "node s")["spanId"]);
    assert!(end(call) >= start(call));
    assert!(
        end(call) <= end(named(&spans, "run")),
        "cancelled spans must nest inside the run"
    );
    assert!(end(named(&spans, "node s")) <= end(named(&spans, "run")));
}
