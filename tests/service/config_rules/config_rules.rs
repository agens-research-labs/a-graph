use crate::common::cli::a_graph;

/// The service starts only with all five of `port`, `data_dir`, `allowed_hosts`, `allowed_env` and `allowed_ambient`, and no other key: otherwise it ends with 1 and says every error.
#[tokio::test]
async fn config_rules() {
    let path = std::path::Path::new(file!()).parent().unwrap().join("service.yml");
    let r = a_graph(&["--serve", &path.to_string_lossy()], None).await;
    assert_eq!(r.code, 1);
    for wanted in ["data_dir", "allowed_hosts", "allowed_env", "allowed_ambient", "colour"] {
        assert!(r.stderr.contains(wanted), "no `{wanted}` in {}", r.stderr);
    }
}
