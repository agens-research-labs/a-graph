use super::*;

/// The key comes from the environment and never shows.
#[test]
fn key_from_env() {
    let at = Path::root().key("models").idx(3);
    let err = Mistral::new(&api_model("A_GRAPH_TEST_MISTRAL_NOT_SET"), &at).unwrap_err();
    assert_eq!(err.rule, Rule::EnvMissing);
    assert_eq!(err.path.to_string(), "models[3].api_key");
    // SAFETY: a variable only this test uses.
    unsafe { std::env::set_var("A_GRAPH_TEST_MISTRAL_SECRET", "super-secret-value") };
    let shown = format!("{:?}", Mistral::new(&api_model("A_GRAPH_TEST_MISTRAL_SECRET"), &at).unwrap());
    assert!(!shown.contains("super-secret-value") && shown.contains("redacted"), "{shown}");
}
