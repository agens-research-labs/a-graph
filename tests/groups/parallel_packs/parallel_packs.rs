use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A parallel group hands the packed answers of its branches to out.
#[tokio::test]
async fn parallel_packs() {
    let (p, seen) = program(here(), "parallel_packs", echo).await;
    let out = p.run("hi").await.unwrap();
    // the final output follows the edges, not the order the branches finished
    let ids: Vec<String> = items(&out).into_iter().map(|(id, _)| id).collect();
    assert_eq!(ids, ["s", "i", "d", "e", "f", "z"]);
    assert_eq!(asked_of(&seen, 'F'), D_AND_E);
    assert_eq!(items(&out)[5].1, format!("Z(F({D_AND_E}))"));
}
