use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!())
        .parent()
        .unwrap()
        .join(std::path::Path::new(file!()).file_stem().unwrap())
        .with_extension("yml")
        .to_string_lossy()
        .to_string()
}

/// `output: stdout` is honored: the log is on the standard output, with the result.
#[tokio::test]
async fn log_on_stdout() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "x"], None).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout.contains("\"code\":\"run.start\"") && r.stdout.contains("\"items\""),
        "{}",
        r.stdout
    );
    assert!(!r.stderr.contains("run.start"));
}
