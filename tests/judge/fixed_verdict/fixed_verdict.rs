use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The judge answers the fixed verdict.
#[tokio::test]
async fn fixed_verdict() {
    let (p, seen) = program(here(), "fixed_verdict", approved()).await;
    p.run("hi").await.unwrap();
    let judge = seen.lock().unwrap().iter().map(body_of).find(|b| b["format"]["properties"]["validation"].is_object()).unwrap();
    assert_eq!(judge["format"]["required"], json!(["validation", "reasons", "log"]));
    assert_eq!(judge["messages"][0]["content"], "Judge the text.");
}
