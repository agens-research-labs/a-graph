use crate::common::cli::a_graph;

/// `session_retention_seconds` and `session_max_bytes` are at least 1: the service does not start with 0, nor with
/// a negative value.
#[tokio::test]
async fn config_storage() {
    for (file, key) in [
        ("retention.yml", "session_retention_seconds"),
        ("max_bytes.yml", "session_max_bytes"),
        ("retention_negative.yml", "session_retention_seconds"),
        ("max_bytes_negative.yml", "session_max_bytes"),
    ] {
        let path = std::path::Path::new(file!()).parent().unwrap().join(file);
        let r = a_graph(&["--serve", &path.to_string_lossy()], None).await;
        assert_eq!(r.code, 1, "{key}");
        assert!(r.stderr.contains(key), "{}", r.stderr);
        assert!(r.stderr.contains("bad_value"), "{}", r.stderr);
    }
}
