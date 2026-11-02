use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Two calls in one round come back in order.
#[tokio::test]
async fn two_calls_in_order() {
    let calls = vec![("t", json!({ "city": "Roma" })), ("t", json!({ "city": "Milano" }))];
    let (p, _) = program(here(), "two_calls_in_order", model(calls, false)).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(
        items(&out)[0].1,
        r#"R({"got":{"city":"Roma"}}|{"got":{"city":"Milano"}})"#
    );
}
