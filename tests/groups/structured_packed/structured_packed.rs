use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Structured answers of the branches are packed as a list of from node and content.
#[tokio::test]
async fn structured_packed() {
    let (p, seen) = program(here(), "structured_packed", echo).await;
    let out = p.run("hi").await.unwrap();
    let packed: Value = serde_json::from_str(&asked_of(&seen, 'F')).unwrap();
    assert_eq!(packed, json!([
        { "from_node": "d", "content": { "v": "D(I(S(hi)))" } },
        { "from_node": "e", "content": { "v": "E(I(S(hi)))" } },
    ]));
    // d and e sent their JSON to the final output too
    assert!(matches!(&out.items[2].message, Some(Content::Json(_))));
}
