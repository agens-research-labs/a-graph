use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The count starts again at each appearance of a node.
#[tokio::test]
async fn count_restarts() {
    // max_iter is 1 and the node n is twice in the list: each time it asks once
    let (p, _) = program(here(), "count_restarts", model(city(), false)).await;
    let out = p.run("hi").await.unwrap();
    let answers: Vec<String> = items(&out).into_iter().filter(|(id, _)| id == "n").map(|(_, t)| t).collect();
    assert_eq!(answers.len(), 2);
}
