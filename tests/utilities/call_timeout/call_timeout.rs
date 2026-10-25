use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn utility() -> Utility {
    manager(&own_yml(here())).get("u").unwrap().clone()
}

/// A utility that does not answer within its timeout is stopped and the call fails.
#[tokio::test]
async fn call_timeout() {
    let started = std::time::Instant::now();
    let e = call_utility(&utility(), &UtilityInput::Text("x".into())).await.unwrap_err();
    assert!(e.contains("did not answer within 1 s"), "{e}");
    assert!(started.elapsed().as_secs() < 4);
}
