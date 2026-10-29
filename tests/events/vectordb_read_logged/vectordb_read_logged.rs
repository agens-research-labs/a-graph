use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The reading of a vectordb is an event with the id and how many chunks came back.
#[tokio::test]
async fn vectordb_read_logged() {
    // SAFETY: the name of the variable the yml writes; nothing else reads it.
    unsafe { std::env::set_var("VECTORDB_TEST_KEY", "secret") };
    crate::common::clean_output("target/test-output/vectordb_read_logged.log");
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::ok(r#"{"result":{"points":[{"payload":{"text":"one"}},{"payload":{"text":"two"}}]}}"#)).await;
    let (p, _) = program_with(here(), "vectordb_read_logged", &[("embedding", embedding_port), ("vectordb", vectordb_port)], Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("q")).await.unwrap();
    let lines = crate::common::run::log_lines("vectordb_read_logged");
    let read = lines.iter().find(|l| l["code"] == "read.vectordb").expect("a read.vectordb event");
    assert_eq!((read["id"].as_str(), read["chunks"].as_u64(), read["node"].as_str()), (Some("contracts_db"), Some(2), Some("n")));
}
