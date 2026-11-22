use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join(std::path::Path::new(file!()).file_stem().unwrap())
        .with_extension("yml")
        .to_string_lossy()
        .to_string()
}

/// The judge answers a verdict without `validation`, every other node a text.
fn judge(req: &crate::common::http::Req) -> Resp {
    let body: Value = serde_json::from_str(&req.body).unwrap();
    if body["format"]["properties"]["validation"].is_object() {
        let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
        return Resp::ok(answer(&verdict.to_string()));
    }
    Resp::ok(answer("ok"))
}

/// A run that fails says why on the standard error, where the log goes too, without what made the
/// error.
#[tokio::test]
async fn error_kept_out() {
    let (model, _) = listen(judge).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "hi"], None).await;
    assert_eq!(r.code, 3, "{}", r.stderr);
    assert!(r.stderr.contains("schema_mismatch"), "{}", r.stderr);
    assert!(
        r.stderr.contains("content_redacted"),
        "the log is there too: {}",
        r.stderr
    );
    assert!(!r.stderr.contains("diabetes"), "{}", r.stderr);
}
