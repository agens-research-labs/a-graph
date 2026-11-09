use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// An answer that is not JSON is an error.
#[tokio::test]
async fn not_json() {
    let e = call_utility(&utility(), &UtilityInput::Text("x".into()))
        .await
        .unwrap_err();
    assert!(e.message.contains("utility `u`") && e.message.contains("JSON"), "{e}");
    assert!(!e.message.contains("hello"), "{e}");
    assert!(e.detail.unwrap().contains("hello"), "what it wrote is the detail");
}
