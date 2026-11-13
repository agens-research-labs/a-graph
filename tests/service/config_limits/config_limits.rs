use crate::common::cli::a_graph;

/// `max_runs`, `run_timeout_seconds`, `read_timeout_seconds` and `max_reading_per_address` are at least 1: the
/// service does not start with 0.
#[tokio::test]
async fn config_limits() {
    for key in [
        "max_runs",
        "run_timeout_seconds",
        "read_timeout_seconds",
        "max_reading_per_address",
    ] {
        let path = std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join(format!("{key}.yml"));
        let r = a_graph(&["--serve", &path.to_string_lossy()], None).await;
        assert_eq!(r.code, 1, "{key}");
        assert!(r.stderr.contains(key), "{}", r.stderr);
    }
}
