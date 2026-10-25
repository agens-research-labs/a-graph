use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An image media goes as base64.
#[tokio::test]
async fn image_base64() {
    let (p, seen) = program(here(), "image_base64", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("what is it")).await.unwrap();
    // "png-bytes" in base64
    assert_eq!(sent(&seen)["messages"][0]["images"], json!(["cG5nLWJ5dGVz"]));
}
