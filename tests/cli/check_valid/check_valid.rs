use crate::common::cli::*;

fn path() -> String {
    std::path::Path::new(file!()).parent().unwrap().join(std::path::Path::new(file!()).file_stem().unwrap()).with_extension("yml").to_string_lossy().to_string()
}

/// `--check` on a valid file writes nothing and ends with 0.
#[tokio::test]
async fn check_valid() {
    let r = a_graph(&["--check", &ported(&path(), &[])], None).await;
    assert_eq!((r.code, r.stdout.as_str(), r.stderr.as_str()), (0, "", ""));
}
