use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group without out passes nothing and what follows waits for the branches.
#[tokio::test]
async fn group_without_out() {
    let (p, seen) = program(here(), "group_without_out", echo).await;
    let out = p.run("hi").await.unwrap();
    let order = order(&seen);
    // the two branches run in some order, then `end`
    assert!(order == "SIDEZ" || order == "SIEDZ", "{order}");
    assert!(order.starts_with("SI") && order.ends_with('Z'));
    assert_eq!(items(&out).last().unwrap(), &pair("z", "Z(hi)"));
}
