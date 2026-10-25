use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// A text goes in and a text comes out.
#[tokio::test]
async fn text_roundtrip() {
    let out = call_utility(&utility(), &UtilityInput::Text("hello world".into())).await.unwrap();
    assert_eq!(out, UtilityOutput::Text("HELLO WORLD".into()));
}
