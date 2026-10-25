use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!()).parent().unwrap().join(std::path::Path::new(file!()).file_stem().unwrap()).with_extension("yml").to_string_lossy().to_string()
}

/// `--check` on an invalid file ends with 1 and says every error on the standard error, none on the standard output.
#[tokio::test]
async fn check_invalid() {
    let r = a_graph(&["--check", &path()], None).await;
    assert_eq!(r.code, 1);
    assert!(r.stdout.is_empty());
    assert!(r.stderr.contains("graph.nodes[0].model.id") && r.stderr.contains("unknown_ref"), "{}", r.stderr);
}
