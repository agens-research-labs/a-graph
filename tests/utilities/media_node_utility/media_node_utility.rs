use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media from the node before reaches a program as its bytes: no file is written.
#[tokio::test]
async fn media_node_utility() {
    crate::common::run::set_env();
    let (p, seen) = program(here(), "media_node_utility", Resp::ok(chat("ok", None))).await;
    let given = vec![a_graph::models::MediaOutput {
        mime_type: "image/png".into(),
        data: "AAAAAAAA".into(),
        id: Some("cover".into()),
    }];
    p.run_node(
        p.nodes.entity("n").unwrap(),
        NodeInput {
            user_message: Some("q".into()),
            agent_media: given,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let content = sent(&seen)["messages"][1]["content"].as_str().unwrap().to_string();
    assert_eq!(
        content, "data [AAAAAAAA], type [image/png], path []\n\nQ: q",
        "the utility gets the bytes and the type, no path"
    );
}
