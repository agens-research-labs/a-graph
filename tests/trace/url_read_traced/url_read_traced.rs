use crate::common::run_node::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The download of a media and of a prompt is an interval of its own, inside the run, with its bytes; the url is
/// never in the trace.
#[tokio::test]
async fn url_read_traced() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (files, _) = listen(|r| {
        if r.path.starts_with("/doc.md") {
            Resp::ok("# the document")
        } else {
            Resp::ok("content:\n  system: \"Be brief.\"\n  user: \"Q: {{user_message}}\"\n")
        }
    })
    .await;
    let (p, _) = program_with(
        here(),
        "url_read_traced",
        &[("collector", collector_port), ("files", files)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    Arc::new(p).run("why").await.unwrap();
    let spans = spans_of(&collector);
    let run = named(&spans, "run")["spanId"].clone();
    for (name, bytes) in [("read media doc", 14), ("read prompt pr", 61)] {
        let s = named(&spans, name);
        assert_eq!(s["parentSpanId"], run, "{name} hangs from the run");
        assert_eq!(
            attribute(s, "a_graph.bytes"),
            Some(json!({ "intValue": bytes.to_string() })),
            "{s}"
        );
    }
    assert!(
        !collector.lock().unwrap().iter().any(|r| r.body.contains("secret")),
        "the url is never in the trace"
    );
}
