use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `from: message` the utility gets the text of the node and answers a file; the file has the id of the media and goes where its flags say.
#[tokio::test]
async fn media_from_message() {
    let (p, _) = program(here(), "media_from_message", Resp::ok(chat("the report", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.assistant.message, Some(Content::Text("the report".into())), "the text goes on as it is");
    assert_eq!(out.assistant.media.len(), 1);
    let file = &out.assistant.media[0];
    assert_eq!((file.id.as_deref(), file.mime_type.as_str(), file.data.as_str()), (Some("report"), "application/pdf", "AAAA"));
    assert!(out.node.media.is_empty(), "`node` is false for this media");
}
