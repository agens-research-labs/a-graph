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
    let e = call_utility(&utility(), &UtilityInput::Text("x".into())).await.unwrap_err();
    assert!(e.contains("utility `u`") && e.contains("JSON"), "{e}");
}
