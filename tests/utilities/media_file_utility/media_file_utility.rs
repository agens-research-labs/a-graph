use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The utility gets the path of the file and its type, and the text it answers is what the model reads.
#[tokio::test]
async fn media_file_utility() {
    let (p, seen) = program(here(), "media_file_utility", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("what is it")).await.unwrap();
    let content = sent(&seen)["messages"][1]["content"].as_str().unwrap().to_string();
    assert_eq!(content, "converted contract.pdf application/pdf 9 bytes\n\nQ: what is it");
}
