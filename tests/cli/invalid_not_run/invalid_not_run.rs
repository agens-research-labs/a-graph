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

/// An invalid file is not run: it ends with 1 and nothing reaches the model.
#[tokio::test]
async fn invalid_not_run() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "x"], None).await;
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty() && seen.lock().unwrap().is_empty());
}
