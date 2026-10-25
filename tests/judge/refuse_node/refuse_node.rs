use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A refused message with on error node goes to the next node as the problem.
#[tokio::test]
async fn refuse_node() {
    let (p, seen) = program(here(), "refuse_node", refused()).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(order(&seen), "SJZE");
    let problem = "---\nfrom agent judge\n---\nreasons: too long\nlog: checked the length";
    assert_eq!(asked(&seen, 'Z'), format!("Z({problem})"));
    assert_eq!(items(&out).len(), 3, "the run reached the end");
}
