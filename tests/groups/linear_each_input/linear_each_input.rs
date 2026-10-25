use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A linear group runs its branches one after the other each with the input node output.
#[tokio::test]
async fn linear_each_input() {
    let (p, seen) = program(here(), "linear_each_input", echo).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(order(&seen), "SIDEFZ");
    // e does not take the answer of d: both took the output of i
    assert_eq!(asked_of(&seen, 'F'), D_AND_E);
    assert_eq!(items(&out).len(), 6);
}
