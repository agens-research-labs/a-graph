use crate::common::cli::*;

/// A file that does not exist, no file at all, and an option that does not exist end with 2.
#[tokio::test]
async fn usage_errors() {
    let missing = a_graph(&["tests/cli/usage_errors/does_not_exist.yml", "x"], None).await;
    assert_eq!(missing.code, 2);
    assert!(missing.stderr.contains("cannot read") && missing.stdout.is_empty(), "{}", missing.stderr);

    let nothing = a_graph(&[], None).await;
    assert_eq!(nothing.code, 2);

    let unknown = a_graph(&["--nope", "x.yml"], None).await;
    assert_eq!(unknown.code, 2);
}
