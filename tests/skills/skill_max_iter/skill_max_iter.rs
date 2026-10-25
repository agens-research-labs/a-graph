use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Reading files and running scripts count together against the max iter of the skill.
#[tokio::test]
async fn skill_max_iter() {
    let calls = vec![("read_skill_file", json!({ "skill": "pdf", "path": "references/forms.md" })), ("run_skill_script", json!({ "skill": "pdf", "path": "scripts/extract.sh", "args": [] }))];
    let (p, seen) = program(here(), "skill_max_iter", model(calls, true)).await;
    let e = p.run("go").await.unwrap_err();
    assert_eq!((e.rule.code(), e.path.to_string()), ("tool_limit", "node(n)".to_string()));
    assert!(e.message.contains("skill `pdf`") && e.message.contains("2 times"), "{}", e.message);
    // two requests to the model: the first round (two calls) went through, the second was refused before anything ran
    assert_eq!(seen.lock().unwrap().len(), 2);
}
