use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program on stdio gets the arguments and its result reaches the model.
#[tokio::test]
async fn stdio_roundtrip() {
    let (p, seen) = program(here(), "stdio_roundtrip", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    // the model was told about the tool, with its parameters
    let first = body_of(&seen.lock().unwrap()[0]);
    assert_eq!(first["tools"][0]["function"]["name"], "t");
    assert_eq!(first["tools"][0]["function"]["parameters"]["required"], json!(["city"]));
    // the tool received the arguments and answered them back; the graph sees only the final answer
    assert_eq!(
        items(&out)[0],
        ("n".to_string(), r#"R({"got":{"city":"Roma"}})"#.to_string())
    );
    assert_eq!(
        seen.lock().unwrap().len(),
        3,
        "two calls of the node and one of the end node"
    );
}
