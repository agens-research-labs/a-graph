use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The server is the same process for the whole run.
#[tokio::test]
async fn one_process_per_run() {
    let (p, _) = program(here(), "one_process_per_run", model(ask("count", json!({})), false)).await;
    let out = p.run("go").await.unwrap();
    let answers: Vec<String> = items(&out).into_iter().filter(|(id, _)| id == "n").map(|(_, t)| t).collect();
    assert_eq!(answers, [r#"R({"calls":1})"#, r#"R({"calls":2})"#]);
}
