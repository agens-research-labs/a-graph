use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program that cannot start is an error with the reason.
#[tokio::test]
async fn cannot_start() {
    let u = manager(&own_yml(here())).get("u").unwrap().clone();
    let e = call_utility(&u, &UtilityInput::Text("x".into())).await.unwrap_err();
    assert!(e.contains("utility `u`") && e.contains("cannot start"), "{e}");
}
