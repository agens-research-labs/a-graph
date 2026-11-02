use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

const BASE: &str = "http://127.0.0.1:8000/api/v2/tenants/default_tenant/databases/default_database/collections";
const COLLECTION: &str = "a_graph_live_leases";

/// A real Chroma server (`pip install chromadb`, then `chroma run --port 8000`) and a real Ollama with
/// `embeddinggemma`: the collection is filled through the REST API with the vectors of a-graph, then
/// a node reads it and the chunk about the lease comes first.
#[tokio::test]
#[ignore = "needs a Chroma server on 127.0.0.1:8000 and a local Ollama with embeddinggemma"]
async fn live_chroma() {
    let (p, seen) = program(here(), "live_chroma", Resp::ok(chat("ok", None))).await;

    let documents = [
        "The lease lasts twelve months and renews every year.",
        "Cats sleep most of the day on the sofa.",
        "The tax declaration is due in November.",
    ];
    let vectors = p.embeddings.embed("emb", &documents.map(String::from)).await.unwrap();

    let client = reqwest::Client::new();
    let _ = client.delete(format!("{BASE}/{COLLECTION}")).send().await;
    let created: Value = client
        .post(BASE)
        .json(&json!({ "name": COLLECTION, "get_or_create": true }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"]
        .as_str()
        .expect("Chroma did not create the collection")
        .to_string();
    let added = client
        .post(format!("{BASE}/{id}/add"))
        .json(&json!({ "ids": ["1", "2", "3"], "embeddings": vectors, "documents": documents }))
        .send()
        .await
        .unwrap();
    assert!(added.status().is_success(), "{}", added.text().await.unwrap());

    p.run_node(entity(&p), user("How long does the lease last?"))
        .await
        .unwrap();
    let prompt = sent(&seen)["messages"][1]["content"].as_str().unwrap().to_string();
    assert!(
        prompt.starts_with("Context: The lease lasts twelve months and renews every year."),
        "{prompt}"
    );
    let chunks = prompt
        .trim_start_matches("Context: ")
        .split(" / Question:")
        .next()
        .unwrap();
    assert_eq!(chunks.split("\n\n").count(), 2, "top_k is 2: {prompt}");

    let _ = client.delete(format!("{BASE}/{COLLECTION}")).send().await;
}
