use super::*;

/// Keys come from the environment and never show.
#[test]
fn keys_from_env() {
    let at = Path::root().key("models").idx(1);
    drop(keys()); // sets the variables of the keys that exist
    let missing = BedrockCredentials::Keys { access_key: "A_GRAPH_TEST_AWS_AK".into(), secret_key: "A_GRAPH_TEST_AWS_NOT_SET".into(), session_token: None };
    let err = Bedrock::new(&model(missing), &at).unwrap_err();
    assert_eq!(err.rule, Rule::EnvMissing);
    assert_eq!(err.path.to_string(), "models[1].credentials.secret_key.env");

    let shown = format!("{:?}", Bedrock::new(&model(keys()), &at).unwrap());
    assert!(!shown.contains("secret-test-key") && !shown.contains("AKIATEST") && shown.contains("redacted"), "{shown}");

    // ambient credentials need nothing at load
    assert!(Bedrock::new(&model(BedrockCredentials::Ambient), &at).is_ok());
}
