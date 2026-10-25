use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The produced text is changed once; the final output gets it changed.
#[tokio::test]
async fn output_once() {
    let (p, _) = program(here(), "output_once", Resp::ok(chat("answer", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.assistant.message, Some(Content::Text("[answer]".into())));
}
