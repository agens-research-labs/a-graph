use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!()).parent().unwrap().join(std::path::Path::new(file!()).file_stem().unwrap()).with_extension("yml").to_string_lossy().to_string()
}

/// Without a message argument, the standard input is the message (the last end of line is not part of it).
#[tokio::test]
async fn run_stdin() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)])], Some("a question\non two lines\n")).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(body["messages"][1]["content"], "Q: a question\non two lines");
}
