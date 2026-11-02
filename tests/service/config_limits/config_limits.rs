use crate::common::cli::a_graph;

/// `max_runs` and `run_timeout_seconds` are at least 1: the service does not start with 0.
#[tokio::test]
async fn config_limits() {
    for key in ["max_runs", "run_timeout_seconds"] {
        let path = std::path::Path::new(file!()).parent().unwrap().join(format!("{key}.yml"));
        let r = a_graph(&["--serve", &path.to_string_lossy()], None).await;
        assert_eq!(r.code, 1, "{key}");
        assert!(r.stderr.contains(key), "{}", r.stderr);
    }
}
