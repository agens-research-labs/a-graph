use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The `tenant` and `database` of the yml are the ones in the paths asked to Chroma.
#[tokio::test]
async fn chroma_tenant_db() {
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, db) = listen(|r| match r.method.as_str() {
        "GET" => Resp::ok(r#"{"id":"c-9"}"#),
        _ => Resp::ok(r#"{"documents":[["one"]]}"#),
    })
    .await;
    let (p, _) = program_with(here(), "chroma_tenant_db", &[("vectordb", vectordb_port), ("embedding", embedding_port)], Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("q")).await.unwrap();
    let db = db.lock().unwrap();
    assert_eq!(db[0].path, "/api/v2/tenants/acme/databases/sales/collections/contracts");
    assert_eq!(db[1].path, "/api/v2/tenants/acme/databases/sales/collections/c-9/query");
}
