use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A concurrent group with limit one goes one branch at a time.
#[tokio::test]
async fn concurrent_one() {
    let (p, seen) = program(here(), "concurrent_one", echo).await;
    p.run("hi").await.unwrap();
    assert_eq!(order(&seen), "SIDEFZ");
    assert_eq!(asked_of(&seen, 'F'), D_AND_E);
}
