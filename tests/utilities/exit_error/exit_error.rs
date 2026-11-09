use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// A utility that fails stops with its id and its exit status; what it wrote is the detail.
#[tokio::test]
async fn exit_error() {
    let e = call_utility(&utility(), &UtilityInput::Text("x".into()))
        .await
        .unwrap_err();
    assert!(
        e.message.contains("utility `u`") && e.message.contains("the program exited with"),
        "{e}"
    );
    assert!(!e.message.contains("boom"), "{e}");
    assert_eq!(e.detail.as_deref(), Some("boom: cannot read the file"));
}
