use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A database that does not answer within the `timeout` of the yml stops the node with an error.
#[tokio::test]
async fn search_timeout() {
    // SAFETY: the name of the variable the yml writes; nothing else reads it.
    unsafe { std::env::set_var("VECTORDB_TEST_KEY", "secret") };
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    // accepts the connection and never answers
    let silent = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let vectordb_port = silent.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut held = Vec::new();
        loop {
            held.push(silent.accept().await.unwrap());
        }
    });
    let (p, _) = program_with(here(), "search_timeout", &[("embedding", embedding_port), ("vectordb", vectordb_port)], Resp::ok(chat("ok", None))).await;
    let started = std::time::Instant::now();
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "io");
    assert!(started.elapsed().as_secs() < 10, "the 60 s default was used instead of the 1 s of the yml");
}
