use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media from the node before is written to a temporary file for the utility, which is removed after.
#[tokio::test]
async fn media_node_utility() {
    crate::common::run::set_env();
    let (p, seen) = program(here(), "media_node_utility", Resp::ok(chat("ok", None))).await;
    let given = vec![a_graph::models::MediaOutput { mime_type: "image/png".into(), data: "AAAAAAAA".into(), id: Some("cover".into()) }];
    p.run_node(p.nodes.entity("n").unwrap(), NodeInput { user_message: Some("q".into()), agent_media: given, ..Default::default() }).await.unwrap();
    let content = sent(&seen)["messages"][1]["content"].as_str().unwrap().to_string();
    assert_eq!(content, "converted 0.png image/png 6 bytes\n\nQ: q", "the temporary file has the right extension and the decoded bytes");
    let left: Vec<_> = std::fs::read_dir(std::env::temp_dir().join(format!("a-graph-{}", std::process::id()))).map(|d| d.flatten().collect()).unwrap_or_default();
    assert!(left.is_empty(), "the temporary file is removed: {left:?}");
}
