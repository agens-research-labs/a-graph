use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file that does not exist or is not text is told to the model.
#[tokio::test]
async fn read_missing_binary() {
    let (p, _) = program(here(), "read_missing", model(read("pdf", "nope.md"), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: the skill has no file `nope.md`)");
    let (p, _) = program(here(), "read_binary", model(read("pdf", "assets/blob.bin"), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(error: `assets/blob.bin` is not text)");
}
