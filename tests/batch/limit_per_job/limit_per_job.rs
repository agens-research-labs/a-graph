use crate::common::run_batch::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Limit is how many requests each job holds.
#[tokio::test]
async fn limit_per_job() {
    let (p, seen, _) = program(here(), "limit_per_job", None).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(job_sizes(&seen), [2, 1], "three nodes, two at a time");
    let ids: Vec<&str> = out.items.iter().map(|i| i.agent_id.as_str()).collect();
    assert_eq!(ids, ["s", "i", "d", "e", "g", "f", "z"]);
}
