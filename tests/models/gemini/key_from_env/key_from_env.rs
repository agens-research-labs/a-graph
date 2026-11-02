use super::*;

/// The key comes from the environment and never shows.
#[test]
fn key_from_env() {
    let at = Path::root().key("models").idx(1);
    let err = Gemini::new(&api_model("A_GRAPH_TEST_GEMINI_NOT_SET"), &at).unwrap_err();
    assert_eq!(err.rule, Rule::EnvMissing);
    assert_eq!(err.path.to_string(), "models[1].api_key");

    // SAFETY: a variable only this test uses.
    unsafe { std::env::set_var("A_GRAPH_TEST_GEMINI_SECRET", "super-secret-value") };
    let shown = format!(
        "{:?}",
        Gemini::new(&api_model("A_GRAPH_TEST_GEMINI_SECRET"), &at).unwrap()
    );
    assert!(
        !shown.contains("super-secret-value") && shown.contains("redacted"),
        "{shown}"
    );
}
