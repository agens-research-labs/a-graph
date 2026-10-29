use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!()).parent().unwrap().join(std::path::Path::new(file!()).file_stem().unwrap()).with_extension("yml").to_string_lossy().to_string()
}

/// With `--chat_mode` the standard output is only `node: text`.
#[tokio::test]
async fn run_chat_mode() {
    let (model, _) = listen(|_| Resp::ok(answer("the lease lasts 12 months"))).await;
    let r = a_graph(&[&ported(&path(), &[("model", model)]), "how long?", "--chat_mode"], None).await;
    assert_eq!((r.code, r.stdout.as_str()), (0, "b: the lease lasts 12 months\n"), "{}", r.stderr);
}
