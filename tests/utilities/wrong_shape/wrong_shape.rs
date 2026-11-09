use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// JSON that has neither `text` nor `mime_type` and `data` is an error that says what was expected.
#[tokio::test]
async fn wrong_shape() {
    let e = call_utility(&utility(), &UtilityInput::Text("x".into()))
        .await
        .unwrap_err();
    assert!(e.message.contains("`text`") && e.message.contains("`mime_type`"), "{e}");
    assert!(e.detail.is_some(), "the answer is the detail");
}
