use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!()).parent().unwrap().join(std::path::Path::new(file!()).file_stem().unwrap()).with_extension("yml").to_string_lossy().to_string()
}

/// The log goes to the standard error by default: the standard output is only the result.
#[tokio::test]
async fn log_on_stderr() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "x"], None).await;
    assert_eq!(r.code, 0, "{}", r.stderr);
    serde_json::from_str::<Value>(&r.stdout).unwrap_or_else(|e| panic!("the standard output is not only the result ({e}): {}", r.stdout));
    assert!(r.stderr.contains("\"code\":\"run.start\"") && r.stderr.contains("\"code\":\"model.call\""), "{}", r.stderr);
}
