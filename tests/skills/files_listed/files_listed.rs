use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The files of the folder are listed when the skill is activated.
#[tokio::test]
async fn files_listed() {
    let (p, _) = program(here(), "list_notes", model(activate("notes"), false)).await;
    let out = p.run("go").await.unwrap();
    // the skill notes has no other file: no list
    assert_eq!(
        items(&out)[0].1,
        "R(<skill_content name=\"notes\">\n# Notes\n\nWrite short bullet points.\n</skill_content>)"
    );
    let (p, _) = program(here(), "list_pdf", model(activate("pdf"), false)).await;
    let out = p.run("go").await.unwrap();
    // `leak.md` is a link that leads out of the folder: not listed
    assert_eq!(items(&out)[0].1, format!("R({PDF})"));
}
