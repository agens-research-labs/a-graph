use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Chroma: the collection is found by name, then queried by id; its documents fill `{{chunks}}`.
#[tokio::test]
async fn chroma_reads() {
    serve_at(19005, |_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (_, db) = serve_at(19006, |r| match r.method.as_str() {
        "GET" => Resp::ok(r#"{"id":"c-123","name":"contracts"}"#),
        _ => Resp::ok(r#"{"ids":[["a","b"]],"documents":[["The rent is 800 euro.","The deposit is two months."]],"distances":[[0.1,0.2]]}"#),
    }).await;
    let (p, seen) = program(here(), "chroma_reads", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("rent?")).await.unwrap();

    let db = db.lock().unwrap();
    let base = "/api/v2/tenants/default_tenant/databases/default_database/collections";
    assert_eq!((db[0].method.as_str(), db[0].path.clone()), ("GET", format!("{base}/contracts")));
    assert_eq!((db[1].method.as_str(), db[1].path.clone()), ("POST", format!("{base}/c-123/query")));
    let body: Value = serde_json::from_str(&db[1].body).unwrap();
    assert_eq!((body["query_embeddings"].clone(), body["n_results"].clone(), body["include"].clone()), (json!([[0.5, 0.25, 1.0]]), json!(3), json!(["documents"])));
    assert_eq!(sent(&seen)["messages"][1]["content"], "Context: The rent is 800 euro.\n\nThe deposit is two months. / Question: rent?");
}
