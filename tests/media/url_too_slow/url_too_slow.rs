use crate::common::http::listen_held;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Downloads that pass 60 seconds stop the run before the first node, as `io` on the url being downloaded. The clock
/// is paused: it moves to the limit as soon as nothing else can go on, so the test does not wait.
#[tokio::test(start_paused = true)]
async fn url_too_slow() {
    let (files, _, _never) = listen_held(|_| true, |_| Resp::ok("# the document")).await;
    let (p, seen) = program_with(here(), "url_too_slow", &[("files", files)], Resp::ok(chat("ok", None))).await;
    let e = Arc::new(p).run("summarise").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string().as_str(), e.message.as_str()),
        (
            "io",
            "resources.media[0].source.url",
            "media `doc`: the downloads of the run passed 60 seconds"
        )
    );
    assert!(seen.lock().unwrap().is_empty(), "the model is never called");
}
