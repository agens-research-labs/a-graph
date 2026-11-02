use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!()).parent().unwrap().join(std::path::Path::new(file!()).file_stem().unwrap()).with_extension("yml").to_string_lossy().to_string()
}

/// The message is the second argument; the result is one JSON document on the standard output.
#[tokio::test]
async fn run_json() {
    let (model, seen) = listen(|_| Resp::ok(answer("the lease lasts 12 months"))).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "how long?"], None).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let out: Value = serde_json::from_str(&r.stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {}", r.stdout));
    assert_eq!(out, json!({ "items": [{ "node": "b", "message": "the lease lasts 12 months", "media": [], "reasoning": [] }] }));
    let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(body["messages"][1]["content"], "Q: how long?");
}
