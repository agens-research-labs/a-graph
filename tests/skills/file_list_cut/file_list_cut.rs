use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A list of more than 200 files is cut and says so.
#[tokio::test]
async fn file_list_cut() {
    big_skill("target/test-output/skills_big_b/pdf");
    let (p, _) = program(here(), "file_list_cut", model(activate("pdf"), false)).await;
    let out = p.run("go").await.unwrap();
    let said = &items(&out)[0].1;
    assert_eq!(said.matches("<file>").count(), 200, "{said}");
    assert!(said.contains("<!-- the list is incomplete -->"));
}
