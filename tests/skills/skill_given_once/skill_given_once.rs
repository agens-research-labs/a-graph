use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A skill is given once in a run of the node.
#[tokio::test]
async fn skill_given_once() {
    let calls = vec![
        ("activate_skill", json!({ "name": "pdf" })),
        ("activate_skill", json!({ "name": "pdf" })),
    ];
    let (p, _) = program(here(), "skill_given_once", model(calls, false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(
        items(&out)[0].1,
        format!("R({PDF}|the skill `pdf` is already loaded above)")
    );
}
