use super::*;

/// The azure key comes from the environment.
#[test]
fn azure_key_env() {
    let creds = AzureCredentials::ApiKey("A_GRAPH_TEST_AZURE_NOT_SET".into());
    let err = Anthropic::azure(&azure_model(creds), &Path::root().key("models").idx(2)).unwrap_err();
    assert_eq!(err.rule, Rule::EnvMissing);
    assert_eq!(err.path.to_string(), "models[2].credentials.api_key");
}
