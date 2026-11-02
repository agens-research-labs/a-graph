use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A worker can be a group called by its in node.
#[tokio::test]
async fn group_worker() {
    let (p, _) = program(
        here(),
        "group_worker",
        script(|n, last| {
            if n == 0 {
                json!({ "done": false, "next_worker": "wi", "input": "gather" })
            } else {
                json!({ "done": true, "answer": last })
            }
        }),
    )
    .await;
    let out = p.run("hi").await.unwrap();
    // the group ran as it always does: in, the branches, out
    assert_eq!(ids(&out), ["s", "wi", "wa", "wb", "wo", "c", "z"]);
    assert_eq!(
        text(&out, "wo"),
        "WO(---\nfrom agent wa\n---\nWA(WI(gather))\n\n---\nfrom agent wb\n---\nWB(WI(gather)))"
    );
    // and what comes out of `out` went back to the controller
    assert_eq!(text(&out, "c"), format!("Observation from wi: {}", text(&out, "wo")));
}
