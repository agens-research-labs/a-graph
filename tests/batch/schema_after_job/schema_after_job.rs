use crate::common::run_batch::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The answers of a batch job are checked against the schema of their node, as a single call is.
#[tokio::test]
async fn schema_after_job() {
    let (p, _, _) = program(here(), "json_ok", None).await;
    let out = p.run("hi").await.unwrap();
    let d = out.items.iter().find(|i| i.agent_id == "d").unwrap();
    assert_eq!(d.message, Some(Content::Json(json!("D(I(S(hi)))"))));

    let (p, _, _) = program(here(), "not_json", None).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("schema_mismatch", "node(d)".to_string())
    );
}
