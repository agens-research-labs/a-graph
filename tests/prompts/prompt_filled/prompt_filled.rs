use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The prompt is filled and the model gets the parameters of the node.
#[tokio::test]
async fn prompt_filled() {
    let (p, seen) = program(here(), "prompt_filled", Resp::ok(chat("ok", None))).await;
    let out = p.run_node(entity(&p), NodeInput { user_message: Some("hello".into()), agent_message: Some("from A".into()), ..Default::default() }).await.unwrap();

    let body = sent(&seen);
    assert_eq!(body["messages"][0], json!({ "role": "system", "content": "Be brief." }));
    assert_eq!(body["messages"][1]["content"], "User said: hello / agent said: from A");
    assert_eq!((body["options"]["temperature"].as_f64(), body["options"]["top_p"].as_f64(), body["options"]["num_predict"].as_u64()), (Some(0.2), Some(0.9), Some(50)));
    assert_eq!(out.assistant.message, Some(Content::Text("ok".into())));
    assert_eq!((out.info.usage.input_tokens, out.info.usage.output_tokens), (7, 3));
}
