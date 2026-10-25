use crate::common::cli::a_graph;

/// `retention_seconds` is at least 1: the service does not start with 0.
#[tokio::test]
async fn config_retention() {
    let path = std::path::Path::new(file!()).parent().unwrap().join("service.yml");
    let r = a_graph(&["--serve", &path.to_string_lossy()], None).await;
    assert_eq!(r.code, 1);
    assert!(r.stderr.contains("retention_seconds"), "{}", r.stderr);
}
