use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The media of the connected node are taken when the node asks for them.
#[tokio::test]
async fn media_from_node() {
    crate::common::run::set_env();
    let (p, seen) = program(here(), "media_from_node", Resp::ok(chat("ok", None))).await;
    let agent_media = vec![a_graph::models::MediaOutput { mime_type: "image/png".into(), data: "AAAA".into(), id: Some("cover".into()) }];
    p.run_node(p.nodes.entity("n").unwrap(), NodeInput { user_message: Some("q".into()), agent_media, ..Default::default() }).await.unwrap();
    assert_eq!(sent(&seen)["messages"][0]["images"], json!(["AAAA"]));
}
