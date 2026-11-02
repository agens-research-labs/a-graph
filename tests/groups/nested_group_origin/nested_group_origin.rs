use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group inside a branch gives its out as the origin.
#[tokio::test]
async fn nested_group_origin() {
    let (p, seen) = program(here(), "nested_group_origin", echo).await;
    let out = p.run("hi").await.unwrap();
    let ids: Vec<String> = items(&out).into_iter().map(|(id, _)| id).collect();
    assert_eq!(ids, ["s", "i", "d", "j", "k", "mm", "f", "z"]);
    assert_eq!(
        asked_of(&seen, 'F'),
        "---\nfrom agent d\n---\nD(I(S(hi)))\n\n---\nfrom agent mm\n---\nM(---\nfrom agent k\n---\nK(J(I(S(hi)))))"
    );
}
