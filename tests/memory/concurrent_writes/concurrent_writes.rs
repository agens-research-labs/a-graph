use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Programs built from the same yml (as the service does for each request) run at once and write
/// the same file: no run fails, the file stays a list of records and keeps every exchange.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn concurrent_writes() {
    let path = "target/test-memory/concurrent_writes.json";
    clean_output(path);
    let (port, _) = listen(|_| Resp::ok(chat("a", None))).await;
    let yml = yml_with(here(), "concurrent_writes", &[("model", port)]);
    let runs: Vec<_> = (0..20)
        .map(|i| {
            let p = build(&yml).unwrap_or_else(|e| panic!("{e}"));
            tokio::spawn(async move { p.run_node(entity(&p), user(&format!("q{i}"))).await })
        })
        .collect();
    for run in runs {
        run.await.unwrap().unwrap();
    }
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut questions: Vec<&str> = records[0]["chat"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["role"] == "user")
        .map(|e| e["content"].as_str().unwrap())
        .collect();
    questions.sort_unstable();
    let mut expected: Vec<String> = (0..20).map(|i| format!("q{i}")).collect();
    expected.sort_unstable();
    assert_eq!(questions, expected);
}
