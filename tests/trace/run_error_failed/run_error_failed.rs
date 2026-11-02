use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A run that fails still sends its trace, with the run marked as failed and why.
#[tokio::test]
async fn run_error_failed() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(here(), "run_error_failed", &[("collector", collector_port)], |_| {
        Resp::status(500, "broken")
    })
    .await;
    assert!(p.run("hi").await.is_err());
    let spans = spans_of(&collector);
    let run = named(&spans, "run");
    assert_eq!(run["status"]["code"], 2);
    assert!(
        run["status"]["message"].as_str().unwrap().contains("model_failed"),
        "{run}"
    );
    // the node that was running when it failed is closed too, marked as interrupted
    assert_eq!(named(&spans, "node s")["status"]["message"], "interrupted");
}
