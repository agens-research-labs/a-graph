use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `detail` adds, for every call, the model, the tokens and the text asked and answered.
#[tokio::test]
async fn detail_shows_calls() {
    let (mut p, _) = program(here(), "detail_shows_calls", echo).await;
    let screen = show(&mut p, VerboseLevel::Detail, true, false);
    p.run("hi").await.unwrap();
    let text = screen.text();
    assert!(text.contains("model m · 1 in / 1 out tokens"), "{text}");
    assert!(text.contains("prompt: S(hi)") && text.contains("answer: S(hi)"), "{text}");
}
