use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The query is embedded, Qdrant is searched with the vector, and the texts found fill `{{chunks}}`.
#[tokio::test]
async fn qdrant_reads() {
    // SAFETY: the name of the variable the yml writes; nothing else reads it.
    unsafe { std::env::set_var("VECTORDB_TEST_KEY", "secret") };
    let (embedding_port, emb) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, db) = listen(|_| Resp::ok(r#"{"result":{"points":[{"id":1,"score":0.9,"payload":{"text":"The lease lasts 12 months."}},{"id":2,"score":0.8,"payload":{"text":"The notice goes by registered mail."}}]}}"#)).await;
    let (p, seen) = program_with(
        here(),
        "qdrant_reads",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    p.run_node(entity(&p), user("how long is the lease?")).await.unwrap();

    let emb = emb.lock().unwrap();
    assert_eq!(
        (
            emb[0].path.as_str(),
            serde_json::from_str::<Value>(&emb[0].body).unwrap()["input"].clone()
        ),
        ("/api/embed", json!(["how long is the lease?"]))
    );
    let db = db.lock().unwrap();
    assert_eq!(
        (db[0].method.as_str(), db[0].path.as_str()),
        ("POST", "/collections/contracts/points/query")
    );
    assert!(db[0].headers.contains("api-key: secret"), "{}", db[0].headers);
    let body: Value = serde_json::from_str(&db[0].body).unwrap();
    assert_eq!(
        (
            body["query"].clone(),
            body["limit"].clone(),
            body["with_payload"].clone()
        ),
        (json!([0.5, 0.25, 1.0]), json!(2), json!(["text"]))
    );
    assert_eq!(
        sent(&seen)["messages"][1]["content"],
        "Context: The lease lasts 12 months.\n\nThe notice goes by registered mail. / Question: how long is the lease?"
    );
}
