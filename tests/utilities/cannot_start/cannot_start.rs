use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program that cannot start is an error with the reason.
#[tokio::test]
async fn cannot_start() {
    let u = manager(&own_yml(here())).get("u").unwrap().clone();
    let e = call_utility(&u, &UtilityInput::Text("x".into())).await.unwrap_err();
    assert!(
        e.message.contains("utility `u`") && e.message.contains("cannot start"),
        "{e}"
    );
    assert_eq!(e.detail, None);
}
