use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A status other than 2xx is an error with the status and the body.
#[tokio::test]
async fn http_error() {
    let (utility_port, _) = listen(|_| Resp::status(500, "the converter is down")).await;
    let u = manager(&yml_with(here(), "http_error", &[("utility", utility_port)]))
        .get("u")
        .unwrap()
        .clone();
    let e = call_utility(&u, &UtilityInput::Text("x".into())).await.unwrap_err();
    assert!(
        e.contains("utility `u`") && e.contains("500") && e.contains("the converter is down"),
        "{e}"
    );
}
