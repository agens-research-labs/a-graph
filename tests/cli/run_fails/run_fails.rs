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

/// A run that fails ends with 3, says why on the standard error and writes nothing on the standard output.
#[tokio::test]
async fn run_fails() {
    let (model, _) = listen(|_| Resp::status(500, "broken")).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "x"], None).await;
    assert_eq!(r.code, 3);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(
        r.stderr.contains("model_failed") && r.stderr.contains("500"),
        "{}",
        r.stderr
    );
}
