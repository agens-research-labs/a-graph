use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group alone, with no `start` and no `end`: its `in` node takes the message of the user, and the
/// result is what its nodes send to the final output.
#[tokio::test]
async fn group_alone_runs() {
    let (p, seen) = program(here(), "group_alone_runs", echo).await;
    let out = p.run("hi").await.unwrap();
    let ids: Vec<String> = items(&out).into_iter().map(|(id, _)| id).collect();
    assert_eq!(ids, ["i", "d", "e", "f"]);
    assert_eq!(items(&out)[0].1, "I(hi)");
    assert_eq!(
        items(&out)[3].1,
        "F(---\nfrom agent d\n---\nD(I(hi))\n\n---\nfrom agent e\n---\nE(I(hi)))"
    );
    assert_eq!(seen.lock().unwrap().len(), 4, "one call for each node");
}
