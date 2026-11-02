use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A sequence passes what each node sends on to the next.
#[tokio::test]
async fn sequence_passes_on() {
    let (p, _) = program(here(), "sequence_passes_on", echo).await;
    let out = p.run("hello").await.unwrap();
    assert_eq!(
        items(&out),
        [
            pair("a", "A(hello)"),
            pair("b", "B(A(hello))"),
            pair("c", "C(B(A(hello)))")
        ]
    );
}
