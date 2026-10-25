use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// A utility that fails stops with its own message and its id.
#[tokio::test]
async fn exit_error() {
    let e = call_utility(&utility(), &UtilityInput::Text("x".into())).await.unwrap_err();
    assert!(e.contains("utility `u`") && e.contains("boom: cannot read the file"), "{e}");
}
