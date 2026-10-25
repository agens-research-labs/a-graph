use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Branches that mix text and structured output are refused.
#[tokio::test]
async fn mixed_output_refused() {
    let (p, _) = program(here(), "mixed_output_refused", echo).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "not_supported");
    assert!(e.message.contains("both text and structured"), "{}", e.message);
}
