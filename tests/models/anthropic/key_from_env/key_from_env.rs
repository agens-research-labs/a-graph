use super::*;

/// The key comes from the environment and never shows.
#[test]
fn key_from_env() {
    let at = Path::root().key("models").idx(2);
    let err = Anthropic::new(&api_model("A_GRAPH_TEST_VAR_THAT_IS_NOT_SET"), &at).unwrap_err();
    assert_eq!(err.rule, Rule::EnvMissing);
    assert_eq!(err.path.to_string(), "models[2].api_key");

    // SAFETY: a variable only this test uses.
    unsafe { std::env::set_var("A_GRAPH_TEST_SECRET_KEY", "super-secret-value") };
    let model = Anthropic::new(&api_model("A_GRAPH_TEST_SECRET_KEY"), &at).unwrap();
    let shown = format!("{model:?}");
    assert!(
        !shown.contains("super-secret-value") && shown.contains("redacted"),
        "{shown}"
    );
}
