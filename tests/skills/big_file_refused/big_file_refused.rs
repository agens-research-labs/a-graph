use crate::common::run_skills::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file larger than the limit is refused to the model.
#[tokio::test]
async fn big_file_refused() {
    big_skill("target/test-output/skills_big_a/pdf");
    let (p, _) = program(here(), "big_file_refused", model(read("pdf", "big.txt"), false)).await;
    let out = p.run("go").await.unwrap();
    assert!(items(&out)[0].1.starts_with("R(error: `big.txt` has 100001 bytes, more than the 100000"), "{}", items(&out)[0].1);
}
